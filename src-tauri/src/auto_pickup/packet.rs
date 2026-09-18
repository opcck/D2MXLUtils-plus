//! Packet builders for auto pickup actions.
//!
//! - Packet 0x16: SCMD_PACKET_16_PIKCUP_ITEM (13 bytes)
//! - Packet 0x2A: SCMD_PACKET_2A_ITEM_TO_CUBE (9 bytes)

/// Builds the 13-byte 0x16 packet to pick up an item from the ground.
/// `to_cursor`: false = directly into inventory/belt, true = onto mouse cursor.
pub fn build_pickup_packet(unit_id: u32, to_cursor: bool) -> [u8; 13] {
    let mut packet = [0u8; 13];
    packet[0] = 0x16; // SCMD_PACKET_16_PIKCUP_ITEM
    packet[1..5].copy_from_slice(&4u32.to_le_bytes()); // dwUnitType: 4 (Item)
    packet[5..9].copy_from_slice(&unit_id.to_le_bytes()); // dwUnitGUID
    packet[9..13].copy_from_slice(&(if to_cursor { 1u32 } else { 0u32 }).to_le_bytes()); // bCursor
    packet
}

/// Builds the 9-byte 0x2A packet to put the item held on cursor into the Horadric Cube.
pub fn build_item_to_cube_packet(cursor_item_id: u32, cube_id: u32) -> [u8; 9] {
    let mut packet = [0u8; 9];
    packet[0] = 0x2A; // SCMD_PACKET_2A_ITEM_TO_CUBE
    packet[1..5].copy_from_slice(&cursor_item_id.to_le_bytes()); // dwItemGUID
    packet[5..9].copy_from_slice(&cube_id.to_le_bytes()); // dwCubeGUID
    packet
}
