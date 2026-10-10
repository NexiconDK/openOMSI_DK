fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let mut lib = omsi_sim::texttex::FontLibrary::new(&root);
    let atlas = lib.load("AL-DFI").expect("font");
    for (text, w, o) in [("15:42", 102, 1), ("Nochem Hbf", 422, 1), ("5 min", 114, 2)] {
        let def = omsi_model::TextTexture { variable: "x".into(), font: "AL-DFI".into(), width: w, height: 50, full_color: false, color: [255.0, 106.0, 0.0], orientation: o, grid: 1 };
        let st = omsi_sim::texttex::TextTextureState::new(def, Some(atlas.clone()));
        let img = st.image(text);
        println!("{text}:");
        for y in 0..50u32 {
            let row: String = (0..(w as u32).min(110)).map(|x| if img[((y * w as u32 + x) * 4 + 3) as usize] > 0 { '#' } else { '.' }).collect();
            println!("{y:2} {row}");
        }
    }
}
