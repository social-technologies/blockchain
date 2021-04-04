# Orbit DB

OrbitDB is a peer-to-peer database meaning that each peer has its own instance of a specific database. A database is replicated between the peers automatically resulting in an up-to-date view of the database upon updates from any peer. That is to say, the database gets pulled to the clients.

This means that each application contains the full database that they're using. This in turn changes the data modeling as compared to client-server model where there's usually one big database for all entries: in OrbitDB, the data should be stored, "partitioned" or "sharded" based on the access rights for that data. For example, in a twitter-like application, tweets would not be saved in a global "tweets" database to which millions of users write concurrently, but rather, each user would have their own database for their tweets. To follow a user, a peer would subscribe to a user's feed, ie. replicate their feed database.

## Database Types

OrbitDB supports multiple data models and as such the developer has a variety of ways to structure data. The Orbit DB pallet focuses on a few types, namely:

- Key-Value
- Log (append-only log)
- Feed (same as log database but entries can be removed)
- Documents (store indexed JSON documents)

Additional types can be added in the future, but these are the types implemented for now.

# Functions

The `pallet-orbit-db` is a runtime module on Social 🌎 Network and is implemented as follows:
