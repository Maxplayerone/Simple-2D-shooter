use cgmath::{point2, Point2};

//works to check gravity
pub fn check_player_gravity_collission(
    player_pos: Point2<f32>,
    block_pos: Point2<f32>,
    quad_size: usize,
    length: usize
) -> Option<Point2<f32>> {

    let fsize = quad_size as f32;
    if player_pos.y <= block_pos.y + fsize && player_pos.y > block_pos.y && player_pos.x > block_pos.x && player_pos.x < (block_pos.x + fsize * length as f32) {
        return Some(point2::<f32>(player_pos.x, block_pos.y + fsize));
    }
    None
}
