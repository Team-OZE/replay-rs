use std::collections::HashMap;
use std::io::{BufRead, Cursor, Read, Seek, SeekFrom};
use flate2::{Decompress, FlushDecompress};
use log::{info, warn};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Serialize};

#[derive(Serialize, FromPrimitive, Debug)]
pub enum SlotColor {
    RED = 1,
    BLUE = 2,
    TEAL = 3,
    PURPLE = 4,
    YELLOW = 5,
    ORANGE = 6,
    GREEN = 7,
    PINK = 8,
    GRAY = 9,
    LIGHTBLUE = 10,
    DARKGREEN = 11,
    BROWN = 12,
    MAROON = 13,
    NAVY = 14,
    TURQUOISE = 15,
    VIOLET = 16,
    WHEAT = 17,
    PEACH = 18,
    MINT = 19,
    LAVENDER = 20,
    COAL = 21,
    SNOW = 22,
    EMERALD = 23,
    PEANUT = 24,
    OBSERVER = 25,
    UNKNOWN = 127
}

#[derive(Serialize, FromPrimitive, Debug)]
pub enum SlotRace {
    HUMAN = 1,
    ORC = 2,
    NIGHTELF = 4,
    UNDEAD = 8,
    RANDOM = 20,
    FIXED = 40,
    UNKNOWN = 127
}

#[derive(Serialize, FromPrimitive, Debug)]
pub enum ComputerAIStrength {
    EASY = 0,
    NORMAL = 1,
    INSANE = 2,
    UNKNOWN = 127
}

#[derive(Serialize, FromPrimitive, Debug)]
pub enum SlotStatus {
    EMPTY = 0,
    CLOSED = 1,
    OCCUPIED = 2,
    UNKNOWN = 127
}

#[derive(Serialize, FromPrimitive, Debug)]
pub enum LeaveReason {
    CONNECTION_CLOSED_BY_REMOTE_GAME = 0x01,
    CONNECTION_CLOSED_BY_LOCAL_GAME = 0x0C,
    UNKNOWN
}

#[derive(Serialize, FromPrimitive, Debug, PartialEq)]
pub enum ActionType {
    PAUSE = 0x01,
    RESUME = 0x02,

    SAVE_GAME = 0x06,
    SAVE_GAME_DONE = 0x07,

    ABILITY_BASIC = 0x10,
    ABILITY_WITH_TARGET_LOCATION = 0x11,
    ABILITY_WITH_TARGET_LOCATION_AND_OBJECT = 0x12,
    ITEM_TRANSFER = 0x13,

    CHANGE_SELECTION = 0x16,
    GROUP_ASSIGN = 0x17,
    GROUP_SELECT = 0x18,

    MINIMAP_SIGNAL = 0x68,

    UNKNOWN
}

#[derive(Serialize, Debug)]
pub struct MapLocation {
    pub x: f32,
    pub y: f32
}

#[derive(Serialize)]
pub struct ReplayMeta {
    pub saving_player_id: u8,
    pub is_saving_player_host: bool,
    pub game_name: String,
    pub map_name: String,
    pub game_creator_battle_tag: String
}

#[derive(Serialize)]
pub struct GameSettings {
    pub game_speed: u8,
    pub vis_hide_terrain: bool,
    pub vis_map_explored: bool,
    pub vis_always_visible: bool,
    pub vis_default: bool,
    pub obs_mode: u8,
    pub teams_together: bool,
    pub fixed_teams: u8,
    pub shared_unit_control: bool,
    pub random_hero: bool,
    pub random_races: bool,
    pub obs_referees: bool
}

#[derive(Serialize)]
pub struct Slot {
    pub player_id: u8,
    pub map_download_percent: u8,
    pub status: SlotStatus,
    pub is_computer: bool,
    pub team_index: u8,
    pub color: SlotColor,
    pub race: SlotRace,
    pub ai_strength: ComputerAIStrength,
    pub handicap_percent: u8
}

#[derive(Serialize, Debug)]
pub struct ReplayPlayer {
    pub battle_tag: String,
    pub leave_reason: LeaveReason,
    pub result_byte: u8,
    pub left_at: u64
}

#[derive(Serialize, Debug)]
pub struct ChatMessage {
    pub sender_player_id: u8,
    pub recipient_slot_number: Option<i8>,
    pub flag: Option<u8>,
    pub message: String,
    pub timestamp: u64
}

#[derive(Serialize)]
pub struct ObjectIDs {
    pub id1: u32,
    pub id2: u32
}

#[derive(Serialize, FromPrimitive, Debug, PartialEq)]
pub enum SelectionMode {
    ADD = 0x01,
    REMOVE = 0x02
}

#[derive(Serialize)]
#[derive(Default)]
pub struct ActionData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<MapLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub savegame_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknownA: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknownB: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknownC: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objects: Option<Vec<ObjectIDs>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability_flags: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sel_mode: Option<SelectionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_obj_id_1: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_obj_id_2: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    item_obj_id_1: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    item_obj_id_2: Option<u32>,
}

#[derive(Serialize)]
pub struct Action {
    pub player_id: u8,
    pub timestamp: u64,
    pub action_type: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ActionData>
}

#[derive(Serialize)]
pub struct Replay {
    pub version: u8,
    pub metadata: ReplayMeta,
    pub game_settings: GameSettings,
    pub slots: Vec<Slot>,
    pub players: HashMap<u8, ReplayPlayer>,
    pub chat: Vec<ChatMessage>,
    pub actions: Vec<Action>
}

fn parse_dword(bytes: &[u8]) -> u32 {
    let mut data: u32 = 0;
    for j in 0u8..4u8 {
        data += 256u32.pow(j as u32) * bytes[j as usize] as u32
    }
    return data;
}

fn parse_word(bytes: &[u8]) -> u16 {
    let mut data: u16 = 0;
    for j in 0u8..2u8 {
        data += 256u16.pow(j as u32) * bytes[j as usize] as u16
    }
    return data;
}

fn cursor_read_dword<T>(cursor: &mut Cursor<T>) -> u32 where T: AsRef<[u8]> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).unwrap();
    return parse_dword(&buf);
}

fn cursor_read_dword_float<T>(cursor: &mut Cursor<T>) -> f32 where T: AsRef<[u8]> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf).unwrap();
    buf.reverse();
    return f32::from_be_bytes(buf);
}

fn cursor_read_word<T>(cursor: &mut Cursor<T>) -> u16 where T: AsRef<[u8]> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf).unwrap();
    return parse_word(&buf);
}

fn cursor_read_nullterminated_string<T>(cursor: &mut Cursor<T>) -> String where T: AsRef<[u8]> {
    let mut string_buf: Vec<u8> = vec![];
    cursor.read_until(0x00, &mut string_buf).unwrap();

    let string = String::from_utf8_lossy(&string_buf[..string_buf.len()-1]);
    return string.to_string()
}

fn cursor_read_string<T>(cursor: &mut Cursor<T>, len: usize) -> String where T: AsRef<[u8]> {
    let mut string_buf: Vec<u8> = vec![0u8; len];
    cursor.read_exact(&mut string_buf).unwrap();
    let string = String::from_utf8_lossy(&string_buf);
    info!("Read string: {:?} {}", string_buf, string);
    return string.to_string();
}

pub fn cursor_read_byte<T>(cursor: &mut Cursor<T>) -> u8 where T: AsRef<[u8]> {
    let mut buf: [u8;1] = [0u8];
    cursor.read_exact(&mut buf).unwrap();
    return buf[0];
}

fn cursor_skip_bytes<T>(cursor: &mut Cursor<T>, n: i64) where T: AsRef<[u8]> {
    cursor.seek(SeekFrom::Current(n)).unwrap();
}

fn cursor_read_ability_itemid<T>(cursor: &mut Cursor<T>) -> String where T: AsRef<[u8]> {
    let item_id: String;
    let item_id_end = cursor_read_word(cursor);
    cursor_skip_bytes(cursor, -4);

    if item_id_end == 0x000D {
        item_id = cursor_read_string(cursor, 2);
        cursor_skip_bytes(cursor, 2);
    }
    else {
        item_id = cursor_read_string(cursor, 4);
    }

    return item_id;
}

fn decode_gamesettings(enc: &Vec<u8>) -> Vec<u8> {
    let mut i = 0;
    let mut mask: u8 = 0;
    let mut dec: Vec<u8> = vec![];
    while enc[i] != 0 {
        if i % 8 == 0 { mask = enc[i]; }
        else {
            if mask & (0x1 << (i%8)) == 0 {
                dec.push(enc[i] - 1)
            }
            else {
                dec.push(enc[i])
            }
        }
        i+=1;
    }
    return dec;
}

fn is_bit_set(byte: u8, i: u8) -> bool {
    return (byte & (1 << i)) != 0
}

fn get_bits_value(byte: u8, bits: &[u8]) -> u8 {
    let mut i: u8 = 0;
    let mut s: u8 = 0;
    while i < bits.len() as u8 {
        if is_bit_set(byte, bits[i as usize]) {
            s += 2_u8.pow(i as u32)
        }
        i+=1;
    }
    return s;
}

impl Replay {
    pub fn from_bytes(bytes: &[u8]) -> Replay {
        let mut reader = Cursor::new(bytes);
        info!("Total bytes length: {:?}", bytes.len());
        let mut header: [u8; 48] = [0; 48];
        reader.read_exact(&mut header).unwrap();
        info!("Replay version: {:?}", header);
        let version = header.get(0x0024).unwrap();
        let total_header_length = match version {
            0 => 64,
            1 => 68,
            _ => 68 // Unknown version - try 68
        };

        let mut subheader: Vec<u8> = vec![0; total_header_length - 48];
        reader.read_exact(&mut subheader).unwrap();

        let mut i: u32 = total_header_length as u32;
        let mut k = 0;
        let num_data_blocks = parse_dword(&header[44..48]);
        info!("Total data blocks: {:?}", num_data_blocks);
        let mut block_header: [u8; 12] = [0; 12];
        let mut data: Vec<u8> = vec![];

        while k < num_data_blocks {
            // 3.0 [Data block header]
            match reader.read_exact(&mut block_header) {
                Ok(_) => {
                    let block_data_length_bytes: &[u8] = block_header.get(0..4).unwrap();
                    let block_data_length_inflated_bytes: &[u8] = block_header.get(4..8).unwrap();
                    let block_data_length = parse_dword(block_data_length_bytes);
                    let block_data_length_inflated = parse_dword(block_data_length_inflated_bytes);

                    let crc_deflated = parse_word(block_header.get(8..10).unwrap());
                    let crc_inflated = parse_word(block_header.get(10..12).unwrap());
                    let mut decoder = Decompress::new(true);

                    info!("Word at offset {:#06x} ({:?}) {:?} ({:?}) / inflated: {:?} ({:?})", i, i, block_data_length_bytes, block_data_length, block_data_length_inflated_bytes, block_data_length_inflated);

                    let mut block_data: Vec<u8> = vec![0; block_data_length as usize];
                    match reader.read_exact(&mut block_data) {
                        Ok(_) => {
                            info!("Read datablock of length {:?}.", block_data_length);

                            let mut out: Vec<u8> = Vec::with_capacity(block_data_length_inflated as usize);

                            // 4.0 [Decompressed data]
                            decoder.decompress_vec(&block_data, &mut out, FlushDecompress::Sync).unwrap();
                            decoder.reset(true);
                            info!("Decompressed block length: {:?} / begins with {:?}", out.len(), out.get(0..8).unwrap());

                            data.append(&mut out);
                        }
                        Err(_) => {
                            warn!("Failed to read datablock of length {:?}.", block_data_length);
                        }
                    };
                    i += block_data_length + 12;
                    k+=1;
                }
                Err(_) => break
            }
        }


        info!("Finished replay decoding. Total decoded data length: {:?}", data.len());
        info!("Data starts with {:?}", data.get(0..128).unwrap());

        // Decoding of the actual data

        let mut cursor = Cursor::new(&data);


        // 4.1 [PlayerRecord]
        let player_is_host = cursor_read_byte(&mut cursor) == 0x00;
        let player_id = cursor_read_byte(&mut cursor);

        // Something new - undocumented
        cursor_skip_bytes(&mut cursor, 4);

        let player_name = cursor_read_nullterminated_string(&mut cursor);
        info!("Player name: {:?}", player_name);

        let additional_data_size_byte = cursor_read_byte(&mut cursor);
        cursor_skip_bytes(&mut cursor, additional_data_size_byte as i64);


        // 4.2 [GameName]
        let game_name = cursor_read_nullterminated_string(&mut cursor);
        info!("Game name: {:?}", game_name);

        // There seems to be an additional NUL byte
        cursor_skip_bytes(&mut cursor, 1);

        // 4.3 [Encoded String]
        let mut encoded_gamesettings_buf: Vec<u8> = vec![];
        cursor.read_until(0x00, &mut encoded_gamesettings_buf).unwrap();

        let game_settings_buf = decode_gamesettings(&encoded_gamesettings_buf);
        info!("Decoded gamesettings: {:?}", game_settings_buf);

        // 4.4 [GameSettings]
        let game_speed = get_bits_value(game_settings_buf[0], [0, 1].as_ref());
        let vis_hide_terrain = get_bits_value(game_settings_buf[1], [0].as_ref()) == 1;
        let vis_map_explored = get_bits_value(game_settings_buf[1], [1].as_ref()) == 1;
        let vis_always_visible = get_bits_value(game_settings_buf[1], [2].as_ref()) == 1;
        let vis_default = get_bits_value(game_settings_buf[1], [3].as_ref()) == 1;
        let obs_mode = get_bits_value(game_settings_buf[1], [4, 5].as_ref());
        let teams_together = get_bits_value(game_settings_buf[1], [6].as_ref()) == 1;

        let fixed_teams = get_bits_value(game_settings_buf[2], [1,2].as_ref());
        let shared_unit_control = get_bits_value(game_settings_buf[3], [0].as_ref()) == 1;
        let random_hero = get_bits_value(game_settings_buf[3], [1].as_ref()) == 1;
        let random_races = get_bits_value(game_settings_buf[3], [2].as_ref()) == 1;
        let obs_referees = get_bits_value(game_settings_buf[3], [6].as_ref()) == 1;

        // 4.5 [Map&CreatorName]
        let mut subcursor = Cursor::new(game_settings_buf[13..].as_ref());
        let map_name = cursor_read_nullterminated_string(&mut subcursor);
        let game_creator_name = cursor_read_nullterminated_string(&mut subcursor);

        // 4.6 [PlayerCount]
        let num_players_slots = cursor_read_dword(&mut cursor);

        // 4.7 [GameType]
        let game_type = cursor_read_byte(&mut cursor);
        let is_private_custom_game = cursor_read_byte(&mut cursor);
        cursor_skip_bytes(&mut cursor, 2);

        // 4.8 [LanguageID?]
        cursor_skip_bytes(&mut cursor, 4);

        // 4.9 [PlayerList]
        let mut player_list: HashMap<u8, ReplayPlayer> = HashMap::new();
        player_list.insert(player_id,
                           ReplayPlayer {
                               battle_tag: player_name.clone(),
                               leave_reason: LeaveReason::UNKNOWN,
                               result_byte: 0,
                               left_at: 0,
                           }
        );
        let mut next_record_id = cursor_read_byte(&mut cursor);
        while next_record_id == 0x00 || next_record_id == 0x16 {
            let cur_player_id = cursor_read_byte(&mut cursor);
            // cursor_skip_bytes(&mut cursor, 4);;
            let cur_player_name = cursor_read_nullterminated_string(&mut cursor);
            let additional_data_size_byte = cursor_read_byte(&mut cursor);
            cursor_skip_bytes(&mut cursor, additional_data_size_byte as i64);
            player_list.insert(cur_player_id, ReplayPlayer {
                battle_tag: cur_player_name,
                leave_reason: LeaveReason::UNKNOWN,
                result_byte: 0,
                left_at: 0,
            });
            next_record_id = cursor_read_byte(&mut cursor);
        }
        info!("Loaded player list: {:?}", player_list);

        // Reforged player metadata
        while next_record_id == 0x39 {
            let cur_record_subtype = cursor_read_byte(&mut cursor);
            let cur_record_data_length = cursor_read_dword(&mut cursor);

            cursor_skip_bytes(&mut cursor, cur_record_data_length as i64);
            // TODO: Maybe parse this data too

            next_record_id = cursor_read_byte(&mut cursor);
        }

        // 4.10 [GameStartRecord]
        if next_record_id != 0x19 {
            let mut buf = [0u8; 128];
            cursor.read_exact(&mut buf).unwrap();
            panic!("GameStartRecord did not follow PlayerList: next record id = {:?}. Following bytes: {:?}", next_record_id, buf)
        }

        let data_length = cursor_read_word(&mut cursor);
        let count_slotrecords = cursor_read_byte(&mut cursor);
        let mut i = 0u8;

        let mut slots: Vec<Slot> = Vec::with_capacity(count_slotrecords as usize);

        while i < count_slotrecords {
            let cur_slot_player_id = cursor_read_byte(&mut cursor);
            let cur_slot_map_download_percent = cursor_read_byte(&mut cursor);
            let status_byte = cursor_read_byte(&mut cursor);
            let cur_slot_status = SlotStatus::from_u8(status_byte)
                .or(Option::from(SlotStatus::UNKNOWN))
                .unwrap();
            let cur_slot_is_computer_player = cursor_read_byte(&mut cursor) == 1;
            let cur_slot_team_index = cursor_read_byte(&mut cursor);
            let color_byte = cursor_read_byte(&mut cursor);
            let cur_slot_color =
                SlotColor::from_u8(color_byte + 1)
                    .or(Option::from(SlotColor::UNKNOWN))
                    .unwrap();
            let race_byte = cursor_read_byte(&mut cursor);
            let cur_slot_player_race =
                SlotRace::from_u8(race_byte)
                    .or(Option::from(SlotRace::UNKNOWN))
                    .unwrap();
            let cur_slot_player_computer_ai_strenth =
                ComputerAIStrength::from_u8(cursor_read_byte(&mut cursor))
                    .or(Option::from(ComputerAIStrength::UNKNOWN))
                    .unwrap();
            let cur_slot_handicap_percent = cursor_read_byte(&mut cursor);

            info!("Player slot record read: pid = {:?} status = {:?} is_comp = {:?} team = {:?} color = {:?} ({:?}) race = {:?} ({:?})",
                cur_slot_player_id, cur_slot_status, cur_slot_is_computer_player, cur_slot_team_index, cur_slot_color, color_byte, cur_slot_player_race, race_byte);

            slots.push(Slot {
                player_id: cur_slot_player_id,
                map_download_percent: cur_slot_map_download_percent,
                status: cur_slot_status,
                is_computer: cur_slot_is_computer_player,
                team_index: cur_slot_team_index,
                color: cur_slot_color,
                race: cur_slot_player_race,
                ai_strength: cur_slot_player_computer_ai_strenth,
                handicap_percent: cur_slot_handicap_percent,
            });

            i+=1;
        }

        let random_seed = cursor_read_dword(&mut cursor);
        info!("Random seed: {:?}", random_seed);
        let selection_mode = cursor_read_byte(&mut cursor);
        info!("Selection mode: {:?}", selection_mode);
        let start_spot_count = cursor_read_byte(&mut cursor);
        info!("Start spots count: {:?}", start_spot_count);

        // 5.0 [ReplayData]

        // 0x17 LeaveGame
        let from_index = cursor.position();
        let mut next_record_id = cursor_read_byte(&mut cursor);
        let mut chat: Vec<ChatMessage> = vec![];
        let mut current_timestamp: u64 = 0;
        let mut records: HashMap<u8, u64> = HashMap::new();
        let mut action_records: HashMap<u8, u64> = HashMap::new();
        let mut actions: Vec<Action> = vec![];
        let mut last_leaver_index: u8 = 0;

        loop {
            // info!("Position {:?}, record {:?}", cursor.position() - 1, next_record_id);
            match next_record_id {
                0x17 => {
                    let leave_reason_byte = cursor_read_dword(&mut cursor);
                    let cur_leave_reason = LeaveReason::from_u32(leave_reason_byte).or(Option::from(LeaveReason::UNKNOWN)).unwrap();
                    let cur_player_id = cursor_read_byte(&mut cursor);
                    let cur_result = cursor_read_dword(&mut cursor);
                    cursor_skip_bytes(&mut cursor, 4);

                    info!("{:?} {:?}", cur_leave_reason, cur_result);
                    player_list.entry(cur_player_id).and_modify(|r| {
                        r.leave_reason = cur_leave_reason;
                        r.result_byte = cur_result as u8;
                    }
                    );
                    last_leaver_index = cur_player_id;
                },
                0x1A => {
                    cursor_skip_bytes(&mut cursor, 4);
                },
                0x1B => {
                    cursor_skip_bytes(&mut cursor, 4);
                },
                0x1C => {
                    cursor_skip_bytes(&mut cursor, 4);
                },
                0x1E | 0x1F => {
                    let mut len_following = cursor_read_word(&mut cursor);
                    let increment = cursor_read_word(&mut cursor);
                    // info!("Time increment: {:?}", increment);
                    current_timestamp += increment as u64;
                    len_following -= 2;
                    let total_len_following = len_following.clone();
                    let cursor_position_before_data_read = cursor.position();

                    if len_following > 3 {
                        loop {
                            let cur_action_player_id = cursor_read_byte(&mut cursor);
                            let cur_action_blocks_length = cursor_read_word(&mut cursor);
                            len_following -= 3;

                            player_list.entry(cur_action_player_id).and_modify(|x| x.left_at = current_timestamp);

                            let position_before_read = cursor.position();
                            let mut cur_read_bytes = 0;
                            while cur_read_bytes < cur_action_blocks_length {
                                let cur_position_before_read = cursor.position();

                                let cur_action_id = cursor_read_byte(&mut cursor);
                                if !action_records.contains_key(&cur_action_id)  {
                                    action_records.insert(cur_action_id, 1);
                                }
                                else {
                                    action_records.entry(cur_action_id).and_modify(|x| { *x += 1; });
                                }

                                let mut action = Action {
                                    player_id: cur_action_player_id,
                                    action_type: ActionType::from_u8(cur_action_id).or(Option::from(ActionType::UNKNOWN)).unwrap(),
                                    timestamp: current_timestamp,
                                    data: None,
                                };

                                match cur_action_id {
                                    0x01 => {},
                                    0x02 => {},
                                    0x03 => {
                                        let new_game_speed = cursor_read_byte(&mut cursor);
                                    },
                                    0x04 => {},
                                    0x05 => {},
                                    0x06 => {
                                        let savegame_name = cursor_read_nullterminated_string(&mut cursor);
                                        action.data = Option::from(ActionData {
                                            savegame_name: Option::from(savegame_name),
                                            ..Default::default()
                                        })
                                    },
                                    0x07 => {
                                        cursor_skip_bytes(&mut cursor, 4);
                                    },
                                    0x10 => {
                                        let flags = cursor_read_word(&mut cursor);

                                        cursor_skip_bytes(&mut cursor, 2);
                                        let item_id = cursor_read_ability_itemid(&mut cursor);

                                        let unk_a = cursor_read_dword(&mut cursor);
                                        let unk_b = cursor_read_dword(&mut cursor);

                                        action.data = Option::from(ActionData {
                                            item_id: Option::from(item_id.chars().rev().collect::<String>()),
                                            ability_flags: Option::from(flags),
                                            unknownA: Option::from(unk_a),
                                            unknownB: Option::from(unk_b),
                                            ..Default::default()
                                        })
                                    },
                                    0x11 => {
                                        let flags = cursor_read_word(&mut cursor);

                                        cursor_skip_bytes(&mut cursor, 2);
                                        let item_id = cursor_read_ability_itemid(&mut cursor);

                                        let unk_a = cursor_read_dword(&mut cursor);
                                        let unk_b = cursor_read_dword(&mut cursor);

                                        let loc_x = cursor_read_dword_float(&mut cursor);
                                        let loc_y = cursor_read_dword_float(&mut cursor);

                                        action.data = Option::from(ActionData {
                                            item_id: Option::from(item_id.chars().rev().collect::<String>()),
                                            ability_flags: Option::from(flags),
                                            location: Option::from(MapLocation {
                                                x: loc_x,
                                                y: loc_y
                                            }),
                                            unknownA: Option::from(unk_a),
                                            unknownB: Option::from(unk_b),
                                            ..Default::default()
                                        })
                                    },
                                    0x12 => {
                                        let flags = cursor_read_word(&mut cursor);

                                        cursor_skip_bytes(&mut cursor, 2);
                                        let item_id = cursor_read_ability_itemid(&mut cursor);

                                        let unk_a = cursor_read_dword(&mut cursor);
                                        let unk_b = cursor_read_dword(&mut cursor);

                                        let loc_x = cursor_read_dword_float(&mut cursor);
                                        let loc_y = cursor_read_dword_float(&mut cursor);

                                        let obj_1 = cursor_read_dword(&mut cursor);
                                        let obj_2 = cursor_read_dword(&mut cursor);

                                        action.data = Option::from(ActionData {
                                            item_id: Option::from(item_id.chars().rev().collect::<String>()),
                                            ability_flags: Option::from(flags),
                                            location: Option::from(MapLocation {
                                                x: loc_x,
                                                y: loc_y
                                            }),
                                            unknownA: Option::from(unk_a),
                                            unknownB: Option::from(unk_b),
                                            target_obj_id_1: Option::from(obj_1),
                                            target_obj_id_2: Option::from(obj_2),
                                            ..Default::default()
                                        })
                                    },
                                    0x13 => {
                                        let flags = cursor_read_word(&mut cursor);

                                        cursor_skip_bytes(&mut cursor, 2);
                                        let item_id = cursor_read_ability_itemid(&mut cursor);

                                        let unk_a = cursor_read_dword(&mut cursor);
                                        let unk_b = cursor_read_dword(&mut cursor);

                                        let loc_x = cursor_read_dword_float(&mut cursor);
                                        let loc_y = cursor_read_dword_float(&mut cursor);

                                        let obj_1 = cursor_read_dword(&mut cursor);
                                        let obj_2 = cursor_read_dword(&mut cursor);

                                        let item_obj_1 = cursor_read_dword(&mut cursor);
                                        let item_obj_2 = cursor_read_dword(&mut cursor);

                                        action.data = Option::from(ActionData {
                                            item_id: Option::from(item_id.chars().rev().collect::<String>()),
                                            ability_flags: Option::from(flags),
                                            location: Option::from(MapLocation {
                                                x: loc_x,
                                                y: loc_y
                                            }),
                                            unknownA: Option::from(unk_a),
                                            unknownB: Option::from(unk_b),
                                            target_obj_id_1: Option::from(obj_1),
                                            target_obj_id_2: Option::from(obj_2),
                                            item_obj_id_1: Option::from(item_obj_1),
                                            item_obj_id_2: Option::from(item_obj_2),
                                            ..Default::default()
                                        })
                                    },
                                    0x14 => {
                                        cursor_skip_bytes(&mut cursor, 43);
                                    },
                                    0x16 => {
                                        let select_mode_byte = cursor_read_byte(&mut cursor);
                                        let num_units = cursor_read_word(&mut cursor);
                                        let mut ii: u16 = 0;
                                        let mut objs: Vec<ObjectIDs> = vec![];
                                        while ii < num_units {
                                            objs.push(ObjectIDs {
                                                id1: cursor_read_dword(&mut cursor),
                                                id2: cursor_read_dword(&mut cursor),
                                            });
                                            ii += 1;
                                        }
                                        action.data = Option::from(ActionData {
                                            sel_mode: SelectionMode::from_u8(select_mode_byte),
                                            objects: Option::from(objs),
                                            ..Default::default()
                                        })
                                        // cursor_skip_bytes(&mut cursor, 8*num_units as i64);
                                    },
                                    0x17 => {
                                        let group_num = cursor_read_byte(&mut cursor);
                                        let items_count = cursor_read_word(&mut cursor);
                                        let mut ii: u16 = 0;
                                        let mut objs: Vec<ObjectIDs> = vec![];
                                        while ii < items_count {
                                            objs.push(ObjectIDs {
                                                id1: cursor_read_dword(&mut cursor),
                                                id2: cursor_read_dword(&mut cursor),
                                            });
                                            ii += 1;
                                        }
                                        action.data = Option::from(ActionData {
                                            group_id: Some(group_num),
                                            objects: Option::from(objs),
                                            ..Default::default()
                                        })
                                    },
                                    0x18 => {
                                        cursor_skip_bytes(&mut cursor, 2);
                                    },
                                    0x19 => {
                                        cursor_skip_bytes(&mut cursor, 12);
                                    },
                                    0x1A => {},
                                    0x1B => {
                                        cursor_skip_bytes(&mut cursor, 9);
                                    },
                                    0x1C => {
                                        cursor_skip_bytes(&mut cursor, 9);
                                    },
                                    0x1D => {
                                        cursor_skip_bytes(&mut cursor, 8);
                                    },
                                    0x1E => {
                                        cursor_skip_bytes(&mut cursor, 5);
                                    },
                                    0x21 => {
                                        cursor_skip_bytes(&mut cursor, 8);
                                    },

                                    0x20 => {},
                                    0x22 => {},
                                    0x23 => {},
                                    0x24 => {},
                                    0x25 => {},
                                    0x26 => {},
                                    0x27 => {
                                        cursor_skip_bytes(&mut cursor, 5);
                                    },
                                    0x29 => {},
                                    0x2A => {},
                                    0x2B => {},
                                    0x2C => {},
                                    0x2D => {
                                        cursor_skip_bytes(&mut cursor, 5);
                                    },
                                    0x2E => {
                                        cursor_skip_bytes(&mut cursor, 4);
                                    },
                                    0x2F => {},
                                    0x30 => {},
                                    0x31 => {},
                                    0x32 => {},

                                    0x50 => {
                                        cursor_skip_bytes(&mut cursor, 5);
                                    },
                                    0x51 => {
                                        cursor_skip_bytes(&mut cursor, 9);
                                    },

                                    0x60 => {
                                        let mut buf = vec![];
                                        buf.resize(8, 0);
                                        cursor.read_exact(&mut buf).unwrap();
                                        let command = cursor_read_nullterminated_string(&mut cursor);
                                        info!("Chat command (time {}) (player {}): {} {:?}", current_timestamp, cur_action_player_id, command, buf);

                                        // W3C Replays: Chat messages stored here, but in other replays messages here might shadow chatmessages
                                        if chat.iter().rfind(|el| el.sender_player_id == cur_action_player_id && el.message == command && el.timestamp.abs_diff(current_timestamp) < 500).is_none() {
                                            chat.push(ChatMessage {
                                                message: command,
                                                timestamp: current_timestamp,
                                                flag: None,
                                                recipient_slot_number: None,
                                                sender_player_id: cur_action_player_id
                                            })
                                        }
                                    },
                                    0x61 => {},
                                    0x62 => {
                                        action.data = Option::from(ActionData {
                                            unknownA: Option::from(cursor_read_dword(&mut cursor)),
                                            unknownB: Option::from(cursor_read_dword(&mut cursor)),
                                            unknownC: Option::from(cursor_read_dword(&mut cursor)),
                                            ..Default::default()
                                        })
                                    },
                                    0x66 => {},
                                    0x67 => {},
                                    0x68 => {
                                        let x = cursor_read_dword_float(&mut cursor);
                                        let y = cursor_read_dword_float(&mut cursor);
                                        action.data = Option::from(ActionData {
                                            location: Option::from(MapLocation {
                                                x,
                                                y
                                            }),
                                            ..Default::default()
                                        })
                                    },
                                    0x69 => {
                                        cursor_skip_bytes(&mut cursor, 16);
                                    },
                                    0x6A => {
                                        cursor_skip_bytes(&mut cursor, 16);
                                    },
                                    0x75 => {
                                        cursor_skip_bytes(&mut cursor, 1);
                                    },

                                    // Unknown
                                    0x7a => {
                                        cursor_skip_bytes(&mut cursor, 20);
                                    },
                                    0x7b => {
                                        cursor_skip_bytes(&mut cursor, 16);
                                    },

                                    _ => {
                                        let cur_pos = cursor.position().clone();
                                        let left_bytes = cur_action_blocks_length as u64 + position_before_read - cur_pos;
                                        warn!("({}) Unknown action id: {:#04x}. Read bytes so far: {:?}. Total expected: {:?}", cur_read_bytes, cur_action_id, cur_pos - position_before_read, cur_action_blocks_length);
                                        let mut buf = vec![];
                                        buf.resize(left_bytes as usize, 0);
                                        cursor.read_exact(&mut buf).unwrap();
                                        info!("Following bytes: {:?}", buf);
                                        break;
                                    }
                                }

                                if action.action_type != ActionType::UNKNOWN {
                                    actions.push(action);
                                }

                                let cur_bytes = (cursor.position().clone() - cur_position_before_read) as u16;
                                cur_read_bytes += cur_bytes;
                            }

                            len_following -= (cursor.position() - position_before_read) as u16;

                            if len_following < 1 { break }
                        }
                    }

                    if cursor.position() - cursor_position_before_data_read != total_len_following as u64 {
                        warn!("Mismatch: {:?}/{:?}", cursor.position() - cursor_position_before_data_read, total_len_following);
                    }
                },
                0x20 => {
                    let cur_player_id = cursor_read_byte(&mut cursor);
                    cursor_skip_bytes(&mut cursor, 2);
                    let cur_flag = cursor_read_byte(&mut cursor);
                    let cur_recepient_slotnumber: i8 = (cursor_read_dword(&mut cursor) as i32 - 2) as i8;
                    let cur_message = cursor_read_nullterminated_string(&mut cursor);
                    chat.push(ChatMessage {
                        sender_player_id: cur_player_id,
                        flag: Option::from(cur_flag),
                        recipient_slot_number: Option::from(cur_recepient_slotnumber),
                        message: cur_message,
                        timestamp: current_timestamp
                    })
                },
                0x22 => {
                    cursor_skip_bytes(&mut cursor, 5);
                },
                0x23 => {
                    cursor_skip_bytes(&mut cursor, 10);
                },
                0x2F => {
                    cursor_skip_bytes(&mut cursor, 8);
                },
                0x00 => {
                    info!("Exiting at null. Position: {:?}", cursor.position());
                    break
                }
                _ => {
                    info!("ReplayData: Unknown record id ({:#04x})", next_record_id);
                    break
                }
            }
            if !records.contains_key(&next_record_id) {
                records.insert(next_record_id, 1);
            }
            else {
                records.entry(next_record_id).and_modify(|x| { *x += 1; });
            }
            next_record_id = cursor_read_byte(&mut cursor);
        }
        info!("Records: {:?}", records);
        info!("Action records: {:?}", action_records);

        //
        let mut saving_player_candidate_ids = player_list.keys().filter( |k| match player_list[k].leave_reason {
            LeaveReason::CONNECTION_CLOSED_BY_LOCAL_GAME => true,
            _ => false
        });

        let saving_player_id: Option<&u8> =
            if saving_player_candidate_ids.clone().count() == 1 { Option::from(saving_player_candidate_ids.next()) }
            else { saving_player_candidate_ids.find(|k| player_list[k].battle_tag != "FLO") };

        Replay {
            version: *version,
            metadata: ReplayMeta {
                game_name,
                is_saving_player_host: player_is_host,
                saving_player_id: last_leaver_index,
                map_name,
                game_creator_battle_tag: game_creator_name
            },
            game_settings: GameSettings {
                fixed_teams,
                shared_unit_control,
                random_hero,
                random_races,
                obs_referees,
                vis_default,
                vis_hide_terrain,
                vis_always_visible,
                vis_map_explored,
                teams_together,
                obs_mode,
                game_speed
            },
            slots,
            players: player_list,
            chat,
            actions
        }
    }
}