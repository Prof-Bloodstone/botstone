use crate::commands::{admin::*, config::*, meta::*, owner::*, support::*};
use serenity::framework::standard::macros::group;

// All command groups
// Doesn't currently work as hoped for - see conversation from serenity discord:
// https://discordapp.com/channels/381880193251409931/381912587505500160/754058417420632236
#[group]
#[sub_groups(General, Config, Support, Owner, Admin)]
pub struct All;

#[group]
#[commands(ping, react)]
pub struct General;

#[group]
#[commands(prefix, command, join_role)]
pub struct Config;

#[group]
#[commands(message, reaction_role)]
pub struct Admin;

#[group]
#[commands(support, info)]
pub struct Support;

#[group]
#[commands(quit, _test)]
#[owners_only]
pub struct Owner;
