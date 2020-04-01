# beacon

a p2p app for sharing 1 bit states

a beacon is a keypair and a 1 bit state. the private key holder broadcasts signed state update messages over a p2p network. each message has content:

`pubkey` - the beacon's pubkey

`broadcast_time` - message broadcast time

`beacon_state` - the state of the beacon at the broadcast time

`signature` - signed hash of `broadcast_time` and `beacon_state`

the pubkey is used as the channel name for normal floodsub p2p protocol so peers can watch for certain beacons to change state. only a private key holder can broadcast a state update for a beacon because peers validate messages using the sig and the pubkey.

## user flow

 ### creating and using a beacon
* create beacon
* share beacon
* broadcast beacon state messages

### watching for a beacon
* add beacon to watch list
* update local state according to messages
  * save last message to compare timestamps with new messages 

## features
People will need a place to share beacon pubkeys.

In the app, users get a list of their beacons and a list of the beacons they are watching for. They can add items to each list as well as remove. They can add some local metadata for the beacons e.g. nicknames, colors, pictures, owners.

Settings include message storage, time zone, TTL and rate-limiting, 
