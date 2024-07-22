use gclient::{EventProcessor, GearApi, Result};
use gstd::{prelude::*, ActorId};
use gtest::{Log, Program, ProgramBuilder, System};
use pebbles_game_io::*;
use std::fs;

fn initialize_system() -> System {
    let system = System::new();

    system.init_logger();

    system
}

// fn get_state_binary() -> Vec<u8> {
//     fs::read("target/wasm32-unknown-unknown/debug/pebbles_game.meta.wasm").unwrap()
// }

// fn get_binary() -> Vec<u8> {
//     fs::read("target/wasm32-unknown-unknown/debug/pebbles_game.meta.wasm").unwrap()
// }

/**
 * test init function
 */
#[test]
fn test_init() {
    let system = initialize_system();

    let program = Program::current(&system);

    let id = program.id();
    println!("id:{:?}", id);

    let pebbles_count = 20u32;
    let max_pebbles_per_turn = 5u32;

    program.send(
        2,
        PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: pebbles_count,
            max_pebbles_per_turn: max_pebbles_per_turn,
        },
    );

    let state: GameState = program.read_state(b"").unwrap();

    assert_eq!(state.pebbles_count, pebbles_count);
    assert_eq!(state.max_pebbles_per_turn, max_pebbles_per_turn);
}

/**
 * test handler function
 */
#[test]
fn test_handler() {
    let system = initialize_system();

    let program = Program::current(&system);

    let pebbles_count = 20u32;
    let max_pebbles_per_turn = 5u32;

    //init
    program.send(
        2,
        PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: pebbles_count,
            max_pebbles_per_turn: max_pebbles_per_turn,
        },
    );
    let state: GameState = program.read_state(b"").unwrap();

    assert_eq!(state.pebbles_count, pebbles_count);
    assert_eq!(state.max_pebbles_per_turn, max_pebbles_per_turn);

    // call handler
    let turn_num = 2u32;
    let result = program.send(2u64, PebblesAction::Turn(turn_num));
    // let state_after_handler: GameState = program.read_state(b"").unwrap();
    // assert_eq!(state_after_handler.pebbles_remaining, pebbles_count - turn_num);
    let log = Log::builder().payload(PebblesEvent::CounterTurn(turn_num));
    assert!(!result.main_failed());
    assert!(result.contains(&log));

    let result_giveup = program.send(2u64, PebblesAction::GiveUp);
    assert!(!result_giveup.main_failed());
    assert!(result_giveup.contains(&Log::builder().payload(PebblesEvent::Won(Player::Program))));

    let restart = program.send(
        2u64,
        PebblesAction::Restart {
            difficulty: DifficultyLevel::Hard,
            pebbles_count: 20u32,
            max_pebbles_per_turn: 5u32,
        },
    );
    let state_after_restart: GameState = program.read_state(b"").unwrap();
    assert_eq!(state_after_restart.pebbles_count, 20u32);
    assert!(matches!(
        state_after_restart.difficulty,
        DifficultyLevel::Hard
    ));
}

#[tokio::test]
async fn gclient_test() -> Result<()> {
    let wasm_binary =
        fs::read("target/wasm32-unknown-unknown/debug/pebbles_game.opt.wasm").unwrap();
    let client = GearApi::dev_from_path("target/tmp/gear").await?;
    let mut listener = client.subscribe().await?;
    let mut gas_limit = client
        .calculate_upload_gas(None, wasm_binary.clone(), vec![], 0, true)
        .await?
        .min_limit;

    let (mut message_id, program_id, _) = client
        .upload_program_bytes(
            wasm_binary,
            gclient::now_micros().to_le_bytes(),
            [],
            gas_limit,
            0,
        )
        .await?;

    assert!(listener.message_processed(message_id).await?.succeed());

    let max_pebbles_per_turn = 5u32;
    let pebbles_count = 20u32;
    let init_param = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        max_pebbles_per_turn: max_pebbles_per_turn,
        pebbles_count: pebbles_count,
    };

    gas_limit = client
        .calculate_handle_gas(None, program_id, init_param.encode(), 0, true)
        .await?
        .min_limit;

    (message_id, _) = client
        .send_message(program_id, init_param, gas_limit, 0)
        .await?;
    // let (_, raw_reply, _) = listener.reply_bytes_on(message_id).await?;

    let state: GameState = client.read_state(program_id, vec![]).await?;
    assert_eq!(state.max_pebbles_per_turn, max_pebbles_per_turn);
    assert_eq!(state.pebbles_count, pebbles_count);

    gas_limit = client
        .calculate_handle_gas(None, program_id, PebblesAction::GiveUp.encode(), 0, true)
        .await?
        .min_limit;

    (message_id, _) = client
        .send_message(program_id, PebblesAction::GiveUp, gas_limit, 0)
        .await?;

    let (_, raw_reply, _) = listener.reply_bytes_on(message_id).await?;
    let mut event: PebblesEvent = Decode::decode(
        &mut raw_reply
            .expect("action failed, received an error message instead of a reply")
            .as_slice(),
    )?;
    assert_eq!(PebblesEvent::Won(Player::Program), event);

    let turn_num = 2;
    gas_limit = client
        .calculate_handle_gas(
            None,
            program_id,
            PebblesAction::Turn(turn_num).encode(),
            0,
            true,
        )
        .await?
        .min_limit;

    (message_id, _) = client
        .send_message(program_id, PebblesAction::Turn(turn_num), gas_limit, 0)
        .await?;
    let (_, raw_reply, _) = listener.reply_bytes_on(message_id).await?;
    event = Decode::decode(
        &mut raw_reply
            .expect("action failed, received an error message instead of a reply")
            .as_slice(),
    )?;
    assert_eq!(PebblesEvent::CounterTurn(turn_num), event);

    Ok(())
}
