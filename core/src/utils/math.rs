use crate::imports::*;

pub fn hash_to_y_coord(hash: &tondi_consensus_core::Hash, scale: f64) -> f64 {
    let bytes = hash.as_bytes().iter().take(2).cloned().collect::<Vec<_>>();
    (u16::from_le_bytes(bytes.as_slice().try_into().unwrap()) as f64 - 32767.5) / 32767.5 * scale
}

pub fn bezier(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    steps: usize,
    offset_factor: f64,
) -> Vec<PlotPoint> {
    let mut points = vec![];

    let offset = (x2 - x1) * offset_factor;
    
    // 根据距离调整控制点，使短距离的连接线更直，长距离的连接线更弯曲
    let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let adjusted_offset = if distance < 10.0 {
        offset * 0.3 // 短距离减少弯曲
    } else if distance > 50.0 {
        offset * 1.2 // 长距离增加弯曲
    } else {
        offset
    };

    let control_point1_x = x1 + adjusted_offset;
    let control_point1_y = y1;

    let control_point2_x = x2 - adjusted_offset;
    let control_point2_y = y2;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let inv_t = 1.0 - t;

        let x = x1 * inv_t.powi(3)
            + 3.0 * control_point1_x * inv_t.powi(2) * t
            + 3.0 * control_point2_x * inv_t * t.powi(2)
            + x2 * t.powi(3);
        let y = y1 * inv_t.powi(3)
            + 3.0 * control_point1_y * inv_t.powi(2) * t
            + 3.0 * control_point2_y * inv_t * t.powi(2)
            + y2 * t.powi(3);
        points.push(PlotPoint::new(x, y));
    }

    points
}
