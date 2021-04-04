# Social Bridge Pallet

The Social Bridge is an extensible cross-chain communication protocol. It currently supports bridging between the Social Network blockchain and any other EVM or Substrate based chain. It is an implementation of the Chainsafe ChainBridge.

The bridge pallet or smart contract on each chain forms one side of the bridge. Handler contracts allow for customizable behavior upon receiving transactions to and from the bridge. For example locking up an asset on one side and minting a new one on the other. Its highly customizable - you can deploy a handler contract to perform any action you like.

In its current state ChainBridge operates under a trusted federation model. Deposit events on one chain are detected by a trusted set of off-chain relayers who await finality, submit events to the other chain and vote on submissions to reach acceptance triggering the appropriate handler.

Research is currently underway to reduce the levels of trust required and move toward a fully trust-less bridge.

# Functions

The `pallet-social-bridge` is a runtime module on the Social Network blockchain and is implemented as follows:
