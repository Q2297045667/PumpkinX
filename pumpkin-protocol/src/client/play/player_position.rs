use bytes::BufMut;
use pumpkin_data::packet::clientbound::PLAY_PLAYER_POSITION;
use pumpkin_macros::client_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{ClientPacket, PositionFlag, VarInt, bytebuf::ByteBufMut};

#[client_packet(PLAY_PLAYER_POSITION)]
pub struct CPlayerPosition<'a> {
    teleport_id: VarInt,
    position: Vector3<f64>,
    delta: Vector3<f64>,
    yaw: f32,
    pitch: f32,
    releatives: &'a [PositionFlag],
}

impl<'a> CPlayerPosition<'a> {
    pub fn new(
        teleport_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        releatives: &'a [PositionFlag],
    ) -> Self {
        Self {
            teleport_id,
            position,
            delta,
            yaw,
            pitch,
            releatives,
        }
    }
}

impl ClientPacket for CPlayerPosition<'_> {
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_var_int(&self.teleport_id);
        bytebuf.put_f64(self.position.x);
        bytebuf.put_f64(self.position.y);
        bytebuf.put_f64(self.position.z);
        bytebuf.put_f64(self.delta.x);
        bytebuf.put_f64(self.delta.y);
        bytebuf.put_f64(self.delta.z);
        bytebuf.put_f32(self.yaw);
        bytebuf.put_f32(self.pitch);
        // not sure about that
        bytebuf.put_i32(PositionFlag::get_bitfield(self.releatives));
    }
}
