use cgmath::{point2, Point2};

//works to check gravity
pub fn check_player_gravity_collission(
    player_pos: Point2<f32>,
    block_pos: Point2<f32>,
    quad_size: usize,
) -> Option<Point2<f32>> {

    let _fsize = quad_size as f32;
    if player_pos.y <= block_pos.y + _fsize {
        return Some(point2::<f32>(player_pos.x, block_pos.y + _fsize));
    }
    None
}
