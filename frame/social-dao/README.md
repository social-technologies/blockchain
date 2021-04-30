# Social DAO Pallet

A society can enable its members to benefit in ways that would otherwise be difficult on an individual basis; both individual and social (common) benefits can thus be distinguished, or in many cases found to overlap. A society can also consist of like-minded people governed by their own norms and values within a dominant, larger society. This is sometimes referred to as a subculture, a term used extensively within criminology, and also applied to distinctive subsections of a larger society. A society may be illustrated as an economic, social, industrial or cultural infrastructure, made up of, yet distinct from, a varied collection of individuals.

## Social DAO's

Societies on social.network are collaboration spaces which feature pooling of assets (fungible and non-fungible), and sharing the costs of running and operating a decentralized database powered by OrbitDb and IPFS. Powered by decentralized blockchain, this enables organizations to govern themselves using a governance system of their choosing, using public/private key cryptography for signatures.

## High level overview

1. Creators should be able to create a new social DAO for a Society
   - createSocialDAO()
2. Social token holders should be able to propose spends
   - postProposal(), updateProposal(), deleteProposal()
3. Social token governer should be able to manage heads and execute proposals
   - updateGoverner(), addHead(), removeHead(), executeProposal(), updateBeneficiary(), updateDeposit()
4. Social token head should be able to manage membership
   - addMember(), removeMember(), updateBeneficiary()

## Functions - TODO

The `pallet-social-dao` is a runtime module on the Social Network blockchain and is implemented as follows:

```
createSocialDAO(name, ticker, supply, owner) - anyone can call
```
Stores token[id] = {name, symbol, total, governor, spaces[], proposals[], deposits(enum)}

### Settings functions

```
updateGoverner(newGoverner)
```
If (origin == governer) then token[id].governer = newGoverner

```
updateBeneficiary(newBeneficiary)
```
If (origin == governer) then token[id].beneficiary = newBeneficiary

```
updateProposalDeposit(newProposalDeposit)
```
If (origin == governer) then update token[id].proposalDeposit = newDeposit

```
updateSpaceDeposit(newSpaceDeposit)
```
If (origin == governer) then update token[id].spaceDeposit = newSpaceDeposit

### Treasury proposal functions
```
createProposal(tokenId, metadata, amount, beneficiary)
```
- Require deposit to be paid in tokenId
- Push token[id].proposals[] < {metadata, amount, beneficiary, proposer}

```
updateProposal(tokenId, proposal, metadata, amount, beneficiary)
```
- If (origin == proposer) then edit token[id].proposals[] = {metadata, amount, beneficiary}

```
deleteProposal(tokenId, proposal)
```
- If (origin == proposer && not executed) then delete token[id].proposals[] =

```
executeProposal
```
- if (origin == governer) then get token[id].proposals[proposalId]
- if approved transfer funds to beneficiary and return deposit
- if rejected return deposit to beneficiary
