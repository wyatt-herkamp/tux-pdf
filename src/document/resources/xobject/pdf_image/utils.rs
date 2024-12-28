pub(crate) fn pull_alpha_out_of_rgb(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let mut rgb = Vec::with_capacity(data.len() / 4 * 3);
    let mut alpha = Vec::with_capacity(data.len() / 4);
    for i in (0..data.len()).step_by(4) {
        rgb.push(data[i]);
        rgb.push(data[i + 1]);
        rgb.push(data[i + 2]);
        alpha.push(data[i + 3]);
    }

    (rgb, alpha)
}
