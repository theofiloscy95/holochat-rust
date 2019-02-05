use std::convert::TryFrom;
use hdk::{
    self,
    entry_definition::{
        ValidatingEntryType,
        ValidatingLinkDefinition,
    },
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        cas::content::Address,
        dna::entry_types::Sharing,
        entry::Entry,
        error::HolochainError,
        json::JsonString,
        cas::content::AddressableContent,
    },
    AGENT_ADDRESS,
};

use crate::message;
use crate::utils;

#[derive(Serialize, Deserialize, Debug, Clone, DefaultJson)]
pub struct Channel {
    pub name: String,
    pub description: String,
    pub public: bool,
}

pub fn public_channel_definition() -> ValidatingEntryType {
    entry!(
        name: "public_channel",
        description: "A channel of which anyone can become a member and post",
        sharing: Sharing::Public,
        native_type: Channel,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: |_channel: Channel, _ctx: hdk::ValidationData| {
            Ok(())
        },

        links: [
            agent_channel_link(),
            channel_message_link()
        ]
    )
}

pub fn direct_channel_definition() -> ValidatingEntryType {
    entry!(
        name: "direct_channel",
        description: "A channel to which new members can only be added at creation",
        sharing: Sharing::Public,
        native_type: Channel,

        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: |_channel: Channel, _ctx: hdk::ValidationData| {
            Ok(())
        },

        links: [
            agent_channel_link(),
            channel_message_link()
        ]
    )
}

fn agent_channel_link() -> ValidatingLinkDefinition {
    from!(
        "%agent_id",
        tag: "rooms",
        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },
        validation: |_source: Address, _target: Address, _ctx: hdk::ValidationData| {
            Ok(())
        }
    )
}

fn channel_message_link() -> ValidatingLinkDefinition {
    to!(
        "message",
        tag: "message_in",
        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },
        validation: |_source: Address, _target: Address, _ctx: hdk::ValidationData| {
            Ok(())
        }
    )
}

// public zome functions

pub fn handle_create_channel(
    name: String,
    description: String,
    public: bool,
) -> ZomeApiResult<()> {
    let channel = Channel {
        name,
        description,
        public,
    };

    let entry = match public {
        true => Entry::App("public_channel".into(), channel.into()),
        false => Entry::App("direct_channel".into(), channel.into()),
    };

    let channel_address = hdk::commit_entry(&entry)?;
    hdk::link_entries(&AGENT_ADDRESS, &channel_address, "rooms")?;
    Ok(())
}

pub fn handle_post_message(channel_name: String, message: message::Message) -> ZomeApiResult<()> {
    let channel_address = get_channel_by_name(&channel_name)?.address();
    let message_address = hdk::commit_entry(&Entry::App("message".into(), message.into()))?;
    hdk::link_entries(&channel_address, &message_address, "message_in")?;
    Ok(())
}

pub fn handle_get_my_channels() -> ZomeApiResult<Vec<Channel>> {
    utils::get_links_and_load_type(&AGENT_ADDRESS, "rooms").map(|result| {
        result.into_iter().map(|elem| elem.entry).collect()
    })
}

pub fn handle_get_messages(channel_name: String) -> ZomeApiResult<Vec<message::Message>> {
    let channel_entry = get_channel_by_name(&channel_name)?;
    utils::get_links_and_load_type(&channel_entry.address(), "message_in").map(|result| {
        result.into_iter().map(|elem| elem.entry).collect()
    })
}

pub fn handle_get_channel(channel_name: String) -> ZomeApiResult<Channel> {
    let channel_entry = get_channel_by_name(&channel_name)?;
    match channel_entry {
        Entry::App(_, channel) => {
            Channel::try_from(channel).map_err(|_| {
                ZomeApiError::Internal("Entry is not a valid channel".into())
            })
        },
        _ => Err(ZomeApiError::Internal("could not get channel with that name".to_string()))
    }
}

fn get_channel_by_name(channel_name: &String) -> ZomeApiResult<Entry> {
    let channels = handle_get_my_channels()?;
    channels
    .iter()
    .filter(|f| f.name == *channel_name)
    .map(|channel| match channel.public {
        true => Entry::App("public_channel".into(), channel.into()),
        false => Entry::App("direct_channel".into(), channel.into()),
    })
    .next()
    .ok_or(ZomeApiError::Internal("Could not find channel with name".to_string()))
}


