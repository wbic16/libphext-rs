#[cfg(test)]
mod regressions {

    use crate::phext::{self};
    //use std::{collections::HashMap, io::Write};

    #[test]
    fn helios_stalled_parse() {
        let helios = std::fs::read_to_string("regression-1-helios.phext").expect("Unable to open helios.phext");
        let coord = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let result = phext::fetch(&helios, coord);
        let actual = result.len();
        println!("Helios: {}", actual);
        // The fixture is read from the working tree, so its byte length
        // depends on the checkout's line endings: an LF checkout gives 358,
        // a CRLF (Windows) checkout gives 370 — one extra byte for each of
        // the first scroll's 12 newlines. Both values are correct.
        assert!(
            actual == 358 || actual == 370,
            "unexpected first-scroll length: {actual}"
        );

        let msg = std::fs::read_to_string("regression-1-msg.txt").expect("Unable to find msg.txt");
        let push_coord = phext::to_coordinate("2.1.100/1.1.1/1.1.1");
        let result = phext::replace(helios.as_str(), push_coord, msg.as_str());
        println!("Update: {}", msg.len());
        let actual = result.len();
        // Same line-ending dependence as above: LF gives 15150, CRLF gives
        // 15317 — one extra byte for each of the 167 newlines across the
        // two fixture files (159 in helios, 8 in msg).
        assert!(
            actual == 15150 || actual == 15317,
            "unexpected replaced length: {actual}"
        );
    }
}