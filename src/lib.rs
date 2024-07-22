#![no_std]

use gstd::*;
use pebbles_game_io::*;

static mut PEBBLES_GAME: Option<GameState> = None;

#[no_mangle]
extern "C" fn init() {
    let PebblesInit {
        pebbles_count,
        difficulty,
        max_pebbles_per_turn,
    } = msg::load().expect("Failed to load payload in init function");

    if pebbles_count <= max_pebbles_per_turn {
        msg::reply("errors in params", 0).expect("Failed to reply from `init()`");
        return;
    }

    let state = GameState {
        pebbles_count: pebbles_count,
        max_pebbles_per_turn: max_pebbles_per_turn,
        difficulty: difficulty,
        pebbles_remaining: pebbles_count,
        first_player: Player::User,
        winner: None,
    };
    unsafe { PEBBLES_GAME = Some(state) }
}

#[no_mangle]
extern "C" fn handle() {
    let payload: PebblesAction = msg::load().expect("Failed to load payload in handle function");
    let state = unsafe { PEBBLES_GAME.as_mut().expect("State isn't initialized") };
    match payload {
        PebblesAction::GiveUp => {
            state.winner = Some(Player::Program);
            state.pebbles_remaining = 0;
            let _ = msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to reply from `handle()`");
        }

        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            state.difficulty = difficulty;
            state.pebbles_count = pebbles_count;
            state.max_pebbles_per_turn = max_pebbles_per_turn;
            state.pebbles_remaining = pebbles_count;
            state.winner = Some(Player::Program);
            let _ = msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to reply from `handle()`");
        }

        PebblesAction::Turn(num) => {
            if num > state.max_pebbles_per_turn || num == 0 {
                let _ = msg::reply(PebblesEvent::CounterTurn(num), 0).expect("Failed to reply from `handle()`");
                return;
            }
            state.pebbles_remaining -= num;
            if state.pebbles_remaining == 0 {
                state.winner = Some(Player::User);
                let _ = msg::reply(PebblesEvent::Won(Player::User), 0).expect("Failed to reply from `handle()`");
                return;
            }
       
            let program_remove_count = get_program_remove_num(state.max_pebbles_per_turn, state.pebbles_remaining, state.difficulty.clone());
            state.pebbles_remaining -= program_remove_count;
            if state.pebbles_remaining == 0 {
                state.winner = Some(Player::Program);
                let _ = msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to reply from `handle()`");
            } else {
                let _ = msg::reply(PebblesEvent::CounterTurn(num), 0).expect("Failed to reply from `handle()`");
            }
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let state = unsafe {
        PEBBLES_GAME.take().expect("State isn't initialized")
    };

    msg::reply(state, 0).expect("Failed to reply from `state()`");
}

/**
 *  Here is a algorithm of esay level and hard level
 */
fn get_program_remove_num(max_num: u32, remain_num: u32, level: DifficultyLevel) -> u32 {
    match level {
        DifficultyLevel::Easy => {
            let rand_num = get_random_u32();
            let final_num = if rand_num < max_num {
                rand_num
            } else {
                max_num
            };
            final_num
        },

        DifficultyLevel::Hard => {
            let final_num = if max_num >= remain_num {
                max_num
            } else if remain_num % (max_num + 1u32) > 0 {
                remain_num % (max_num + 1u32)
            } else {
                let rand_num = get_random_u32();
                if rand_num < max_num {
                    rand_num
                } else {
                    max_num
                }
            };
            
            final_num
        }
    }
}

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}
