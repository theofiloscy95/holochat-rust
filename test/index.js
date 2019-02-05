const path = require('path')
const { Config, Container, Scenario } = require('@holochain/holochain-nodejs')
Scenario.setTape(require('tape'))

const dnaPath = path.join(__dirname, "../dist/bundle.json")
const dna = Config.dna(dnaPath, 'app-spec')
const agentAlice = Config.agent("alice")
const agentBob = Config.agent("bob")

const instanceAlice = Config.instance(agentAlice, dna)
const instanceBob = Config.instance(agentBob, dna)

const scenario1 = new Scenario([instanceAlice], {debugLog: false})
const scenario2 = new Scenario([instanceAlice, instanceBob])


scenario2.runTape('agentId', async (t, { alice, bob }) => {
  t.ok(alice.agentId)
  t.notEqual(alice.agentId, bob.agentId)
})



const testNewChannelParams = {
  name: "test new channel",
  description : "testing params",
  public: true
}

const testMessage = {
  timestamp: "100000",
  text : "Some text"
}




scenario1.runTape('Can create a public channel with no other members and retrieve it', async (t, {alice} ) => {
  const create_result = await alice.callSync('chat', 'create_channel', testNewChannelParams)
  console.log(create_result)
  t.notEqual(create_result.Ok, undefined)

  const get_result = alice.call('chat', 'get_my_channels', {})
  console.log(get_result)
  t.deepEqual(get_result.Ok.length, 1)
})


scenario1.runTape('Can post a message to the channel and retrieve', async (t, {alice}) => {
  const create_result = await alice.callSync('chat', 'create_channel', testNewChannelParams)
  console.log(create_result)
  t.notEqual(create_result.Ok, undefined)

  const get_result = alice.call('chat', 'get_my_channels', {})
  console.log(get_result)
  t.deepEqual(get_result.Ok.length, 1)

  const post_result = await alice.callSync('chat', 'post_message', {channel_name: testNewChannelParams.name, message: testMessage})
  console.log(post_result)
  t.notEqual(post_result.Ok, undefined)

  const get_message_result = alice.call('chat', 'get_messages', {channel_name: testNewChannelParams.name, min_count: 10})
  console.log(get_message_result)
  t.deepEqual(get_message_result.Ok[0], testMessage)
})



// scenario2.runTape('scenario test create & publish post -> get from other instance', async (t, {alice, bob}) => {
//   const create_result = await alice.callAsync("chat", "create_channel", testNewChannelParams)
//   t.notEqual(create_result.Ok, undefined)

//   let get_result = await bob.callSync("chat", "get_my_channel", { channel_name: testNewChannelParams.name })
//   t.deepEqual(get_result.Ok, testNewChannelParams);
// })


