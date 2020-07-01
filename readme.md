# remoteDeckel - The first put-it-on-my-tab-bot

## Description:

This project contains the source code of a Telegram-Chat-Bot-Backend, which interacts with the [Telegram Bot Api](https://core.telegram.org/bots/api) in order to perform donations to a local pub.
It works as follows:

- You connect to the bot (via privat chat in Telegram)
- You order drinks (by clicking a button on the chat keyboard), where currently **beer** is a placeholder for everything
- The bot takes your drink-orders and collects them on your virtual/remote tab (Deckel in German)
- You can choose a price per unit
- finally you can donate the collected amount to the account of the connected pub (This part is not implemented yet)

## Background:

This idea has been born during the the current Corona pandemic, where local pubs are struggeling to survive. Friends of mine came up with the idea to donate some amount for every drink we would normally have in a pub, but now we're consuming at home. Which we call: the **remoteDeckel** (remoteTab)!

## Technology:

This repo is currently mainly an experiment around the [Telegram Bot Api](https://core.telegram.org/bots/api), written in [Rust](https://www.rust-lang.org/).
The project contains a `main` function which sets up the bot and it's configurations.
And a `lib`, which contains different serializable types, that can be used in [Telegram](https://telegram.org) communications, database-types and persistence functionality.

This project uses several different rust crates. The main functionality comes from the following:

- [reqwest](https://docs.rs/reqwest/0.10.6/reqwest/) for setting up the webHook, which connects the bot to the Telegam-Bot-Api
- [rocket](https://rocket.rs/) provides routing for incoming Updates/Messages
- [rocket_contrib](https://api.rocket.rs/v0.4/rocket_contrib/) for dealing with JSON-data in requests and responses
- [serde](https://github.com/serde-rs/serde), [serde_json](https://github.com/serde-rs/json) for serialization of data-structures (primarily from/to JSON)
- [serde_yaml](https://docs.serde.rs/serde_yaml/index.html) to read in configuration (like the api_key)
- [diesel](http://diesel.rs/) for dealing with a Postgres database

This project is **NOT a client-library**. If google brought you here while looking for such an implementation, you're most likely actually looking for something like [telebot](https://github.com/bytesnake/telebot).
