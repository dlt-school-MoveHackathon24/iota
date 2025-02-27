// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This example contains code to play the [Tic Tac Toe game](https://github.com/iotaledger/iota/blob/develop/iota_programmability/examples/games/sources/tic_tac_toe.move).
//! Before running this example, the move package needs to be published.
//! Follow the instructions in https://github.com/iotaledger/iota/blob/develop/crates/iota-sdk/README.md#tic-tac-toe-quick-start

use std::{
    io::{stdin, stdout, Write},
    path::PathBuf,
    str::FromStr,
    thread,
    time::Duration,
};

use anyhow::anyhow;
use async_recursion::async_recursion;
use clap::{Parser, Subcommand};
use iota_json_rpc_types::{IotaObjectDataOptions, IotaTransactionBlockResponseOptions};
use iota_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use iota_sdk::{
    json::IotaJsonValue,
    rpc_types::{IotaData, IotaTransactionBlockEffectsAPI},
    types::{
        base_types::{IotaAddress, ObjectID},
        id::UID,
        transaction::Transaction,
    },
    IotaClient, IotaClientBuilder,
};
use iota_types::quorum_driver_types::ExecuteTransactionRequestType;
use serde::Deserialize;
use shared_crypto::intent::Intent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts: TicTacToeOpts = TicTacToeOpts::parse();
    let keystore_path = opts.keystore_path.unwrap_or_else(default_keystore_path);
    let keystore = Keystore::File(FileBasedKeystore::new(&keystore_path)?);

    let game = TicTacToe {
        game_package_id: opts.game_package_id,
        client: IotaClientBuilder::default()
            .build(opts.rpc_server_url)
            .await?,
        keystore,
    };

    match opts.subcommand {
        TicTacToeCommand::NewGame { player_x, player_o } => {
            game.create_game(player_x, player_o).await?;
        }
        TicTacToeCommand::JoinGame {
            my_identity,
            game_id,
        } => {
            game.join_game(game_id, my_identity).await?;
        }
    }

    Ok(())
}

struct TicTacToe {
    game_package_id: ObjectID,
    client: IotaClient,
    keystore: Keystore,
}

impl TicTacToe {
    async fn create_game(
        &self,
        player_x: Option<IotaAddress>,
        player_o: Option<IotaAddress>,
    ) -> Result<(), anyhow::Error> {
        // Default player identity to first and second keys in the keystore if not
        // provided.
        let player_x = player_x.unwrap_or_else(|| self.keystore.addresses()[0]);
        let player_o = player_o.unwrap_or_else(|| self.keystore.addresses()[1]);

        // Create a move call transaction using the TransactionBuilder API.
        let create_game_call = self
            .client
            .transaction_builder()
            .move_call(
                player_x,
                self.game_package_id,
                "shared_tic_tac_toe",
                "create_game",
                vec![],
                vec![
                    IotaJsonValue::from_str(&player_x.to_string())?,
                    IotaJsonValue::from_str(&player_o.to_string())?,
                ],
                None, // The node will pick a gas object belong to the signer if not provided.
                1000000000,
                None,
            )
            .await?;

        // Sign transaction.
        let signature =
            self.keystore
                .sign_secure(&player_x, &create_game_call, Intent::iota_transaction())?;

        // Execute the transaction.

        let response = self
            .client
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(create_game_call, vec![signature]),
                IotaTransactionBlockResponseOptions::full_content(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await?;

        assert!(response.confirmed_local_execution.unwrap());

        // We know `create_game` move function will create 1 object.
        let game_id = response
            .effects
            .as_ref()
            .unwrap()
            .created()
            .first()
            .unwrap()
            .reference
            .object_id;

        println!("Created new game, game id : [{}]", game_id);
        println!("Player X : {}", player_x);
        println!("Player O : {}", player_o);

        self.join_game(game_id, player_x).await?;
        Ok(())
    }

    async fn join_game(
        &self,
        game_id: ObjectID,
        my_identity: IotaAddress,
    ) -> Result<(), anyhow::Error> {
        let game_state = self.fetch_game_state(game_id).await?;
        if game_state.o_address == my_identity {
            println!("You are player O")
        } else if game_state.x_address == my_identity {
            println!("You are player X")
        } else {
            return Err(anyhow!("You are not invited to the game."));
        }
        self.next_turn(my_identity, game_state).await
    }

    #[async_recursion]
    async fn next_turn(
        &self,
        my_identity: IotaAddress,
        game_state: TicTacToeState,
    ) -> Result<(), anyhow::Error> {
        game_state.print_game_board();

        // return if game ended.
        if game_state.game_status != 0 {
            println!("Game ended.");
            match game_state.game_status {
                1 => println!("Player X won!"),
                2 => println!("Player O won!"),
                3 => println!("It's a draw!"),
                _ => {}
            }
            return Ok(());
        }

        if game_state.is_my_turn(my_identity) {
            println!("It's your turn!");
            let row = get_row_col_input(true) - 1;
            let col = get_row_col_input(false) - 1;

            // Create a move call transaction using the TransactionBuilder API.
            let place_mark_call = self
                .client
                .transaction_builder()
                .move_call(
                    my_identity,
                    self.game_package_id,
                    "shared_tic_tac_toe",
                    "place_mark",
                    vec![],
                    vec![
                        IotaJsonValue::from_str(&game_state.info.object_id().to_hex_literal())?,
                        IotaJsonValue::from_str(&row.to_string())?,
                        IotaJsonValue::from_str(&col.to_string())?,
                    ],
                    None,
                    100000000,
                    None,
                )
                .await?;

            // Sign transaction.
            let signature = self.keystore.sign_secure(
                &my_identity,
                &place_mark_call,
                Intent::iota_transaction(),
            )?;

            // Execute the transaction.
            let response = self
                .client
                .quorum_driver_api()
                .execute_transaction_block(
                    Transaction::from_data(place_mark_call, vec![signature]),
                    IotaTransactionBlockResponseOptions::new().with_effects(),
                    Some(ExecuteTransactionRequestType::WaitForLocalExecution),
                )
                .await?;

            assert!(response.confirmed_local_execution.unwrap());

            // Print any execution error.
            let status = response.effects.as_ref().unwrap().status();
            if status.is_err() {
                eprintln!("{:?}", status);
            }
            // Proceed to next turn.
            self.next_turn(
                my_identity,
                self.fetch_game_state(*game_state.info.object_id()).await?,
            )
            .await?;
        } else {
            println!("Waiting for opponent...");
            // Sleep until my turn.
            while !self
                .fetch_game_state(*game_state.info.object_id())
                .await?
                .is_my_turn(my_identity)
            {
                thread::sleep(Duration::from_secs(1));
            }
            self.next_turn(
                my_identity,
                self.fetch_game_state(*game_state.info.object_id()).await?,
            )
            .await?;
        };
        Ok(())
    }

    // Retrieve the latest game state from the server.
    async fn fetch_game_state(&self, game_id: ObjectID) -> Result<TicTacToeState, anyhow::Error> {
        // Get the raw BCS serialised move object data
        let current_game = self
            .client
            .read_api()
            .get_object_with_options(game_id, IotaObjectDataOptions::new().with_bcs())
            .await?;
        current_game
            .object()?
            .bcs
            .as_ref()
            .unwrap()
            .try_as_move()
            .unwrap()
            .deserialize()
    }
}

// Helper function for getting console input
fn get_row_col_input(is_row: bool) -> u8 {
    let r_c = if is_row { "row" } else { "column" };
    print!("Enter {} number (1-3) : ", r_c);
    let _ = stdout().flush();
    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");

    if let Ok(number) = s.trim().parse() {
        if number > 0 && number < 4 {
            return number;
        }
    }
    get_row_col_input(is_row)
}

// Clap command line args parser
#[derive(Parser)]
#[clap(
    name = "tic-tac-toe",
    about = "A Byzantine fault tolerant Tic-Tac-Toe with low-latency finality and high throughput",
    rename_all = "kebab-case"
)]
struct TicTacToeOpts {
    #[clap(long)]
    game_package_id: ObjectID,
    #[clap(long)]
    keystore_path: Option<PathBuf>,
    #[clap(long, default_value = "https://fullnode.devnet.iota.io:443")]
    rpc_server_url: String,
    #[clap(subcommand)]
    subcommand: TicTacToeCommand,
}

fn default_keystore_path() -> PathBuf {
    match dirs::home_dir() {
        Some(v) => v.join(".iota").join("iota_config").join("iota.keystore"),
        None => panic!("Cannot obtain home directory path"),
    }
}

#[derive(Subcommand)]
#[clap(rename_all = "kebab-case")]
enum TicTacToeCommand {
    NewGame {
        #[clap(long)]
        player_x: Option<IotaAddress>,
        #[clap(long)]
        player_o: Option<IotaAddress>,
    },
    JoinGame {
        #[clap(long)]
        my_identity: IotaAddress,
        #[clap(long)]
        game_id: ObjectID,
    },
}

// Data structure mirroring move object `games::shared_tic_tac_toe::TicTacToe`
// for deserialization.
#[derive(Deserialize, Debug)]
struct TicTacToeState {
    info: UID,
    gameboard: Vec<Vec<u8>>,
    cur_turn: u8,
    game_status: u8,
    x_address: IotaAddress,
    o_address: IotaAddress,
}

impl TicTacToeState {
    fn print_game_board(&self) {
        println!("     1     2     3");
        print!("  ┌-----┬-----┬-----┐");
        let mut row_num = 1;
        for row in &self.gameboard {
            println!();
            print!("{} ", row_num);
            for cell in row {
                let mark = match cell {
                    0 => "X",
                    1 => "O",
                    _ => " ",
                };
                print!("|  {}  ", mark)
            }
            println!("|");
            print!("  ├-----┼-----┼-----┤");
            row_num += 1;
        }
        print!("\r");
        println!("  └-----┴-----┴-----┘");
    }

    fn is_my_turn(&self, my_identity: IotaAddress) -> bool {
        let current_player = if self.cur_turn % 2 == 0 {
            self.x_address
        } else {
            self.o_address
        };
        current_player == my_identity
    }
}
