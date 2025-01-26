#[cfg(test)]
mod regressions {

    use crate::phext::{self};
    //use std::{collections::HashMap, io::Write};

    #[test]
    fn helios_stalled_parse() {
        let helios = std::fs::read_to_string("regression-1-helios.phext").expect("Unable to open helios.phext");
        let coord = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let result = phext::fetch(&helios, coord);
        let bytes = 370;
        let expected = result.len();
        println!("Helios: {}", result.len());
        assert_eq!(expected, bytes);

        let msg = std::fs::read_to_string("regression-1-msg.txt").expect("Unable to find msg.txt");
        let push_coord = phext::to_coordinate("2.1.100/1.1.1/1.1.1");
        let result = phext::replace(helios.as_str(), push_coord, msg.as_str());
        println!("Update: {}", msg.len());
        let bytes = 15317;
        let expected = result.len();
        assert_eq!(expected, bytes);
    }
}