# remoteDeckel - The first put-it-on-my-tab-bot

## Description:

This project is in a very early stage! Nothing of the following is yet implemented. But at some point it should use the [Telegram Bot Api](https://core.telegram.org/bots/api), to take your orders and write them on your virtual/remote tab (Deckel in German). It should be possible to assign a value to each order, which than can be donated to a pub of your choosing.

## Background:

This idea has been born during the the current Corona pandemic, where local pubs are struggeling to survive. Friends of mine came up with the idea to donate some amount for every drink we would normally have in a pub, but now where consuming at home. Which we call: the **remoteDeckel** (remoteTab)!

## Technology:

This repo is currently mainly an experiment around the [Telegram Bot Api](https://core.telegram.org/bots/api), written in [Rust](https://www.rust-lang.org/).
The project contains a `main` function which sets up the bot and it's configuration.
And a `lib`, which containes different serializable types, that can be used in [Telegram](https://telegram.org) communications.

This project uses several different rust crates like:

- [reqwest](https://docs.rs/reqwest/0.10.6/reqwest/) for setting up the webHook, which connects the bot to the Telegam-Bot-Api
- [rocket](https://rocket.rs/) provides routing for incoming Updates/Messages
- [rocket_contrib](https://api.rocket.rs/v0.4/rocket_contrib/) for dealing with JSON-data in requests and responses
- [serde](https://github.com/serde-rs/serde), [serde_json](https://github.com/serde-rs/json) for serialization of data-structures (primarily from/to JSON)
- [serde_yaml](https://docs.serde.rs/serde_yaml/index.html) to read in configuration (like the api_key)

This project is **NOT a client-library**. If google brought you here while looking for such an implementation, you're most likely actually looking for something like [telebot](https://github.com/bytesnake/telebot).
