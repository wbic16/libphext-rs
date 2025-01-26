#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use crate::phext::{self, check_for_cowbell, PositionedScroll, BOOK_BREAK, CHAPTER_BREAK, COLLECTION_BREAK, LIBRARY_BREAK, SCROLL_BREAK, SECTION_BREAK, SERIES_BREAK, SHELF_BREAK, VOLUME_BREAK};
    use std::{collections::HashMap, io::Write};

    #[test]
    fn test_coordinate_parsing() {
        let example_coordinate: &str = "9.8.7/6.5.4/3.2.1";
        let test: phext::Coordinate = phext::to_coordinate(example_coordinate);
        let address: String = test.to_string();
        assert_eq!(address, example_coordinate, "Coordinate parsing failed");

        let weird_coordinate = "HOME";
        let test_weird = phext::to_coordinate(weird_coordinate).to_string();
        assert_eq!("1.1.1/1.1.1/1.1.1", test_weird);
    }

    #[test]
    fn test_to_urlencoded() {
        let sample1 = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let result1 = sample1.to_urlencoded();
        assert_eq!(result1, "1.1.1;1.1.1;1.1.1");

        let sample2 = phext::to_coordinate("98.76.54/32.10.1/23.45.67");
        let result2 = sample2.to_urlencoded();
        assert_eq!(result2, "98.76.54;32.10.1;23.45.67");
    }

    fn test_helper(delim_in: u8, data: HashMap<&str, &str>) -> bool {
        let mut index: i32 = 0;
        let mut expect1: &str = "not set";
        let mut expect2: &str = "not set";
        let mut expect3: &str = "not set";
        let mut address1: &str = "not set";
        let mut address2: &str = "not set";
        let mut address3: &str = "not set";
        for x in data.keys() {
            if index == 0 { expect1 = x; address1 = data[x]; index += 1; }
            if index == 1 { expect2 = x; address2 = data[x]; index += 1; }
            if index == 2 { expect3 = x; address3 = data[x]; index += 1; }
        }
        if index < 3 { return false; }

        let buf: Vec<u8> = vec![delim_in];
        let delim: &str = std::str::from_utf8(&buf).unwrap();
        let sample: String = format!("{expect1}{delim}{expect2}{delim}{expect3}");

        let coord1: phext::Coordinate = phext::to_coordinate(address1);
        let coord2: phext::Coordinate = phext::to_coordinate(address2);
        let coord3: phext::Coordinate = phext::to_coordinate(address3);

        let text1: String = phext::fetch(&sample, coord1);
        assert_eq!(text1, expect1, "Fetching text for coord1 failed");

        let text2: String = phext::fetch(&sample, coord2);
        assert_eq!(text2, expect2, "Fetching text for coord2 failed");

        let text3: String = phext::fetch(&sample, coord3);
        assert_eq!(text3, expect3, "Fetching text for coord3 failed");

        return true;
    }

    #[test]
    fn test_scrolls() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Hello World", "1.1.1/1.1.1/1.1.1");
        data.insert("Scroll #2 -- this text will be selected", "1.1.1/1.1.1/1.1.2");
        data.insert("Scroll #3 - this text will be ignored", "1.1.1/1.1.1/1.1.3");

        let result: bool = test_helper(phext::SCROLL_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_sections() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Section A", "1.1.1/1.1.1/1.1.1");
        data.insert("Section B", "1.1.1/1.1.1/1.2.1");
        data.insert("Section C", "1.1.1/1.1.1/1.3.1");

        let result: bool = test_helper(phext::SECTION_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_chapters() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Chapter Alpha", "1.1.1/1.1.1/1.1.1");
        data.insert("Chapter Beta", "1.1.1/1.1.1/2.1.1");
        data.insert("Chapter Gamma", "1.1.1/1.1.1/3.1.1");

        let result: bool = test_helper(phext::CHAPTER_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_books() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Book z1", "1.1.1/1.1.1/1.1.1");
        data.insert("Book Something Else #2", "1.1.1/1.1.2/1.1.1");
        data.insert("Book Part 3", "1.1.1/1.1.3/1.1.1");

        let result: bool = test_helper(phext::BOOK_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_volumes() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Volume 1-1-1", "1.1.1/1.1.1/1.1.1");
        data.insert("Volume 1-2-1", "1.1.1/1.2.1/1.1.1");
        data.insert("Volume 1-3-1", "1.1.1/1.3.1/1.1.1");

        let result: bool = test_helper(phext::VOLUME_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_collections() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Collection 1-1-1", "1.1.1/1.1.1/1.1.1");
        data.insert("Collection 2-1-1", "1.1.1/2.1.1/1.1.1");
        data.insert("Collection 3-1-1", "1.1.1/3.1.1/1.1.1");

        let result: bool = test_helper(phext::COLLECTION_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_series() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Series 1-1-1", "1.1.1/1.1.1/1.1.1");
        data.insert("Series 1-1-2", "1.1.2/1.1.1/1.1.1");
        data.insert("Series 1-1-3", "1.1.3/1.1.1/1.1.1");

        let result: bool = test_helper(phext::SERIES_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_shelves() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Shelf 1-1-1", "1.1.1/1.1.1/1.1.1");
        data.insert("Shelf 1-2-1", "1.2.1/1.1.1/1.1.1");
        data.insert("Shelf 1-3-1", "1.3.1/1.1.1/1.1.1");

        let result: bool = test_helper(phext::SHELF_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_libraries() {
        let mut data: HashMap<&str, &str> = std::collections::HashMap::new();
        data.insert("Library 1-1-1", "1.1.1/1.1.1/1.1.1");
        data.insert("Library 2-1-1", "2.1.1/1.1.1/1.1.1");
        data.insert("Library 3-1-1", "3.1.1/1.1.1/1.1.1");

        let result = test_helper(phext::LIBRARY_BREAK as u8, data);
        assert_eq!(result, true);
    }

    #[test]
    fn test_coordinates_invalid() {
        let c1: phext::Coordinate = phext::to_coordinate("0.0.0/0.0.0/0.0.0"); // invalid
        let c2 = phext::Coordinate {
            z: phext::ZCoordinate{library: 0, shelf: 0, series: 0},
            y: phext::YCoordinate{collection: 0, volume: 0, book: 0},
            x: phext::XCoordinate{chapter: 0, section: 0, scroll: 0}};
        assert_eq!(c1, c2);
        let c1b: bool = c1.validate_coordinate();
        let c2b: bool = c2.validate_coordinate();
        assert_eq!(c1b, false);
        assert_eq!(c2b, false);
    }

    #[test]
    fn test_coordinates_valid() {
        let c1: phext::Coordinate = phext::to_coordinate("1001.254.253/32.4.8/4.2.1"); // valid
        let c2 = phext::Coordinate {
            z: phext::ZCoordinate{library: 1001, shelf: 254, series: 253},
            y: phext::YCoordinate{collection: 32, volume: 4, book: 8},
            x: phext::XCoordinate{chapter: 4, section: 2, scroll: 1}};
        assert_eq!(c1, c2);
        assert_eq!(c1.y.volume, 4);
        let c1b: bool = c1.validate_coordinate();
        let c2b: bool = c2.validate_coordinate();
        assert_eq!(c1b, false);
        assert_eq!(c2b, false);
    }

    #[test]
    fn test_url_encoding() {
        let c1: phext::Coordinate = phext::to_coordinate("142.143.144;145.146.147;148.149.150"); // valid
        let c2 = phext::Coordinate {
            z: phext::ZCoordinate{library: 142, shelf: 143, series: 144},
            y: phext::YCoordinate{collection: 145, volume: 146, book: 147},
            x: phext::XCoordinate{chapter: 148, section: 149, scroll: 150}};
        assert_eq!(c1, c2);
        let c1b: bool = c1.validate_coordinate();
        let c2b: bool = c2.validate_coordinate();
        assert_eq!(c1b, true);
        assert_eq!(c2b, true);

        let c3 = phext::to_coordinate("1001.1002.1003;1004.1005.1006;1007.1008.1009");
        let c4 = phext::Coordinate {
            z: phext::ZCoordinate{library: 1001, shelf: 1002, series: 1003},
            y: phext::YCoordinate{collection: 1004, volume: 1005, book: 1006},
            x: phext::XCoordinate{chapter: 1007, section: 1008, scroll: 1009}
        };
        assert_eq!(c3, c4);
        let c3b = c3.validate_coordinate();
        let c4b = c4.validate_coordinate();
        assert_eq!(c3b, false);
        assert_eq!(c4b, false);
    }

    #[test]
    fn test_realistic_parse() {
        let coord: phext::Coordinate = phext::to_coordinate("6.13.4/2.11.4/2.20.3");
        let subspace = "here's some text at 6.13.4/2.11.4/2.20.3this is the next scroll and won't be picked";
        let result = phext::fetch(subspace, coord);
        assert_eq!(result, "here's some text at 6.13.4/2.11.4/2.20.3");
    }

    #[test]
    fn test_dead_reckoning() {
        let mut test: String = "".to_string();
        test += "random text in 1.1.1/1.1.1/1.1.1 that we can skip past";
        test.push(LIBRARY_BREAK);
        test += "everything in here is at 2.1.1/1.1.1/1.1.1";
        test.push(SCROLL_BREAK);
        test += "and now we're at 2.1.1/1.1.1/1.1.2";
        test.push(SCROLL_BREAK);
        test += "moving on up to 2.1.1/1.1.1/1.1.3";
        test.push(BOOK_BREAK);
        test += "and now over to 2.1.1/1.1.2/1.1.1";
        test.push(SHELF_BREAK);
        test += "woot, up to 2.2.1/1.1.1/1.1.1";
        test.push(LIBRARY_BREAK);
        test += "here we are at 3.1.1/1.1.1.1.1";
        test.push(LIBRARY_BREAK); // 4.1.1/1.1.1/1.1.1
        test.push(LIBRARY_BREAK); // 5.1.1/1.1.1/1.1.1
        test += "getting closer to our target now 5.1.1/1.1.1/1.1.1";
        test.push(SHELF_BREAK); // 5.2.1
        test.push(SHELF_BREAK); // 5.3.1
        test.push(SHELF_BREAK); // 5.4.1
        test.push(SHELF_BREAK); // 5.5.1
        test.push(SERIES_BREAK); // 5.5.2
        test.push(SERIES_BREAK); // 5.5.3
        test.push(SERIES_BREAK); // 5.5.4
        test.push(SERIES_BREAK); // 5.5.5
        test += "here we go! 5.5.5/1.1.1/1.1.1";
        test.push(COLLECTION_BREAK); // 5.5.5/2.1.1/1.1.1
        test.push(COLLECTION_BREAK); // 5.5.5/3.1.1/1.1.1
        test.push(COLLECTION_BREAK); // 5.5.5/4.1.1/1.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.1.2/1.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.1.3/1.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.1.4/1.1.1
        test += "this test appears at 5.5.5/4.1.4/1.1.1";
        test.push(VOLUME_BREAK); // 5.5.5/4.2.1/1.1.1
        test.push(VOLUME_BREAK); // 5.5.5/4.3.1/1.1.1
        test.push(VOLUME_BREAK); // 5.5.5/4.4.1/1.1.1
        test.push(VOLUME_BREAK); // 5.5.5/4.5.1/1.1.1
        test.push(VOLUME_BREAK); // 5.5.5/4.6.1/1.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.1/2.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.1/3.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.1/4.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.1/5.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.6.2/1.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.6.3/1.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.6.4/1.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.6.5/1.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.6.6/1.1.1
        test.push(BOOK_BREAK); // 5.5.5/4.6.7/1.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.7/2.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.7/3.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.7/4.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.7/5.1.1
        test.push(SCROLL_BREAK); // 5.5.5/4.6.7/5.1.2
        test.push(SCROLL_BREAK); // 5.5.5/4.6.7/5.1.3
        test.push(SCROLL_BREAK); // 5.5.5/4.6.7/5.1.4
        test.push(SCROLL_BREAK); // 5.5.5/4.6.7/5.1.5
        test.push(SCROLL_BREAK); // 5.5.5/4.6.7/5.1.6
        test += "here's a test at 5.5.5/4.6.7/5.1.6";
        test.push(SCROLL_BREAK); // 5.5.5/4.6.7/5.1.7
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.7/6.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.7/7.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.7/8.1.1
        test.push(CHAPTER_BREAK); // 5.5.5/4.6.7/9.1.1
        test.push(SECTION_BREAK); // 5.5.5/4.6.7/9.2.1
        test.push(SECTION_BREAK); // 5.5.5/4.6.7/9.3.1
        test.push(SECTION_BREAK); // 5.5.5/4.6.7/9.4.1
        test.push(SECTION_BREAK); // 5.5.5/4.6.7/9.5.1
        test.push(SCROLL_BREAK);  // 5.5.5/4.6.7/9.5.2
        test.push(SCROLL_BREAK);  // 5.5.5/4.6.7/9.5.3
        test.push(SCROLL_BREAK);  // 5.5.5/4.6.7/9.5.4
        test.push(SCROLL_BREAK);  // 5.5.5/4.6.7/9.5.5
        test.push(SCROLL_BREAK);  // 5.5.5/4.6.7/9.5.6
        test.push(SCROLL_BREAK);  // 5.5.5/4.6.7/9.5.7
        test.push(SCROLL_BREAK);  // 5.5.5/4.6.7/9.5.8
        test.push(SCROLL_BREAK);  // 5.5.5/4.6.7/9.5.9
        test += "Expected Test Pattern Alpha Whisky Tango Foxtrot";
        let coord: phext::Coordinate = phext::to_coordinate("5.5.5/4.6.7/9.5.9");
        let result = phext::fetch(&test, coord);
        assert_eq!(result, "Expected Test Pattern Alpha Whisky Tango Foxtrot");

        let coord2 = phext::to_coordinate("5.5.5/4.6.7/5.1.6");
        let result2 = phext::fetch(&test, coord2);
        assert_eq!(result2, "here's a test at 5.5.5/4.6.7/5.1.6");
    }

    #[test]
    fn test_line_break() {
        assert_eq!(phext::LINE_BREAK, '\n');
    }

    #[test]
    fn test_more_cowbell() {
        let test1 = check_for_cowbell("Hello\x07");
        let test2 = check_for_cowbell("nope\x17just more scrolls");
        assert_eq!(phext::MORE_COWBELL, '\x07');
        assert_eq!(test1, true);
        assert_eq!(test2, false);
    }

    #[test]
    fn test_coordinate_based_insert() {
        let mut test: String = "".to_string();
        test += "aaa";               // 1.1.1/1.1.1/1.1.1
        test.push(LIBRARY_BREAK); // 2.1.1/1.1.1/1.1.1
        test += "bbb";               //
        test.push(SCROLL_BREAK);  // 2.1.1/1.1.1/1.1.2
        test += "ccc";

        // append 'ddd' after 'ccc'
        let root = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let coord1 = phext::to_coordinate("2.1.1/1.1.1/1.1.3");
        let expected1 = phext::get_subspace_coordinates(test.as_bytes(), coord1);
        assert_eq!(expected1.2.z.library, 2);
        assert_eq!(expected1.2.z.shelf, 1);
        assert_eq!(expected1.2.z.series, 1);
        assert_eq!(expected1.2.y.collection, 1);
        assert_eq!(expected1.2.y.volume, 1);
        assert_eq!(expected1.2.y.book, 1);
        assert_eq!(expected1.2.x.chapter, 1);
        assert_eq!(expected1.2.x.section, 1);
        assert_eq!(expected1.2.x.scroll, 2);
        assert_eq!(expected1.0, 11);
        assert_eq!(expected1.1, 11);

        let mut expected_coord = phext::default_coordinate();
        expected_coord.z.library = 2;
        expected_coord.x.scroll = 3;
        assert_eq!(coord1, expected_coord);

        let update1 = phext::insert(test, coord1, "ddd");
        assert_eq!(update1, "aaa\x01bbb\x17ccc\x17ddd");

        // append 'eee' after 'ddd'
        let coord2 = phext::to_coordinate("2.1.1/1.1.1/1.1.4");
        let update2 = phext::insert(update1, coord2, "eee");
        assert_eq!(update2, "aaa\x01bbb\x17ccc\x17ddd\x17eee");

        // append 'fff' after 'eee'
        let coord3 = phext::to_coordinate("2.1.1/1.1.1/1.2.1");
        let update3 = phext::insert(update2, coord3, "fff");
        assert_eq!(update3, "aaa\x01bbb\x17ccc\x17ddd\x17eee\x18fff");

        // append 'ggg' after 'fff'
        let coord4 = phext::to_coordinate("2.1.1/1.1.1/1.2.2");
        let update4 = phext::insert(update3, coord4, "ggg");
        assert_eq!(update4, "aaa\x01bbb\x17ccc\x17ddd\x17eee\x18fff\x17ggg");

        // append 'hhh' after 'ggg'
        let coord5 = phext::to_coordinate("2.1.1/1.1.1/2.1.1");
        let update5 = phext::insert(update4, coord5, "hhh");
        assert_eq!(update5, "aaa\x01bbb\x17ccc\x17ddd\x17eee\x18fff\x17ggg\x19hhh");

        // append 'iii' after 'eee'
        let coord6 = phext::to_coordinate("2.1.1/1.1.1/1.1.5");
        let update6 = phext::insert(update5, coord6, "iii");
        assert_eq!(update6, "aaa\x01bbb\x17ccc\x17ddd\x17eee\x17iii\x18fff\x17ggg\x19hhh");

        // extend 1.1.1/1.1.1/1.1.1 with '---AAA'
        let update7 = phext::insert(update6, root, "---AAA");
        assert_eq!(update7, "aaa---AAA\x01bbb\x17ccc\x17ddd\x17eee\x17iii\x18fff\x17ggg\x19hhh");

        // extend 2.1.1/1.1.1/1.1.1 with '---BBB'
        let coord8 = phext::to_coordinate("2.1.1/1.1.1/1.1.1");
        let update8 = phext::insert(update7, coord8, "---BBB");
        assert_eq!(update8, "aaa---AAA\x01bbb---BBB\x17ccc\x17ddd\x17eee\x17iii\x18fff\x17ggg\x19hhh");

        // extend 2.1.1/1.1.1/1.1.2 with '---CCC'
        let coord9 = phext::to_coordinate("2.1.1/1.1.1/1.1.2");
        let update9 = phext::insert(update8, coord9, "---CCC");
        assert_eq!(update9, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd\x17eee\x17iii\x18fff\x17ggg\x19hhh");

        // extend 2.1.1/1.1.1/1.1.3 with '---DDD'
        let coord10 = phext::to_coordinate("2.1.1/1.1.1/1.1.3");
        let update10 = phext::insert(update9, coord10, "---DDD");
        assert_eq!(update10, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee\x17iii\x18fff\x17ggg\x19hhh");

        // extend 2.1.1/1.1.1/1.1.4 with '---EEE'
        let coord11 = phext::to_coordinate("2.1.1/1.1.1/1.1.4");
        let update11 = phext::insert(update10, coord11, "---EEE");
        assert_eq!(update11, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii\x18fff\x17ggg\x19hhh");

        // extend 2.1.1/1.1.1/1.1.5 with '---III'
        let coord12 = phext::to_coordinate("2.1.1/1.1.1/1.1.5");
        let update12 = phext::insert(update11, coord12, "---III");
        assert_eq!(update12, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff\x17ggg\x19hhh");

        // extend 2.1.1/1.1.1/1.2.1 with '---FFF'
        let coord13 = phext::to_coordinate("2.1.1/1.1.1/1.2.1");
        let update13 = phext::insert(update12, coord13, "---FFF");
        assert_eq!(update13, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg\x19hhh");

        // extend 2.1.1/1.1.1/1.2.2 with '---GGG'
        let coord14 = phext::to_coordinate("2.1.1/1.1.1/1.2.2");
        let update14 = phext::insert(update13, coord14, "---GGG");
        assert_eq!(update14, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg---GGG\x19hhh");

        // extend 2.1.1/1.1.1/2.1.1 with '---HHH'
        let coord15 = phext::to_coordinate("2.1.1/1.1.1/2.1.1");
        let update15 = phext::insert(update14, coord15, "---HHH");
        assert_eq!(update15, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg---GGG\x19hhh---HHH");

        // insert 'jjj' at 2.1.1/1.1.2/1.1.1
        let coord16 = phext::to_coordinate("2.1.1/1.1.2/1.1.1");
        let update16 = phext::insert(update15, coord16, "jjj");
        assert_eq!(update16, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg---GGG\x19hhh---HHH\x1Ajjj");

        // insert 'kkk' at 2.1.1/1.2.1/1.1.1
        let coord17 = phext::to_coordinate("2.1.1/1.2.1/1.1.1");
        let update17 = phext::insert(update16, coord17, "kkk");
        assert_eq!(update17, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg---GGG\x19hhh---HHH\x1Ajjj\x1Ckkk");

        // insert 'lll' at 2.1.1/2.1.1/1.1.1
        let coord18 = phext::to_coordinate("2.1.1/2.1.1/1.1.1");
        let update18 = phext::insert(update17, coord18, "lll");
        assert_eq!(update18, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg---GGG\x19hhh---HHH\x1Ajjj\x1Ckkk\x1Dlll");

        // insert 'mmm' at 2.1.2/1.1.1/1.1.1
        let coord19 = phext::to_coordinate("2.1.2/1.1.1/1.1.1");
        let update19 = phext::insert(update18, coord19, "mmm");
        assert_eq!(update19, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg---GGG\x19hhh---HHH\x1Ajjj\x1Ckkk\x1Dlll\x1Emmm");

        // insert 'nnn' at 2.2.1/1.1.1/1.1.1
        let coord20 = phext::to_coordinate("2.2.1/1.1.1/1.1.1");
        let update20 = phext::insert(update19, coord20, "nnn");
        assert_eq!(update20, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg---GGG\x19hhh---HHH\x1Ajjj\x1Ckkk\x1Dlll\x1Emmm\x1Fnnn");

        // insert 'ooo' at 3.1.1/1.1.1/1.1.1
        let coord21 = phext::to_coordinate("3.1.1/1.1.1/1.1.1");
        let update21 = phext::insert(update20, coord21, "ooo");
        assert_eq!(update21, "aaa---AAA\x01bbb---BBB\x17ccc---CCC\x17ddd---DDD\x17eee---EEE\x17iii---III\x18fff---FFF\x17ggg---GGG\x19hhh---HHH\x1Ajjj\x1Ckkk\x1Dlll\x1Emmm\x1Fnnn\x01ooo");
    }

    #[test]
    fn test_coordinate_based_replace() {
        // replace 'AAA' with 'aaa'
        let coord0 = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let update0 = phext::replace("AAA\x17bbb\x18ccc\x19ddd\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj", coord0, "aaa");
        assert_eq!(update0, "aaa\x17bbb\x18ccc\x19ddd\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'bbb' with '222'
        let coord1 = phext::to_coordinate("1.1.1/1.1.1/1.1.2");
        let update1 = phext::replace(update0.as_str(), coord1, "222");
        assert_eq!(update1, "aaa\x17222\x18ccc\x19ddd\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'ccc' with '3-'
        let coord2 = phext::to_coordinate("1.1.1/1.1.1/1.2.1");
        let update2 = phext::replace(update1.as_str(), coord2, "3-");
        assert_eq!(update2, "aaa\x17222\x183-\x19ddd\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'ddd' with 'delta'
        let coord3 = phext::to_coordinate("1.1.1/1.1.1/2.1.1");
        let update3 = phext::replace(update2.as_str(), coord3, "delta");
        assert_eq!(update3, "aaa\x17222\x183-\x19delta\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'eee' with 'a bridge just close enough'
        let coord4 = phext::to_coordinate("1.1.1/1.1.2/1.1.1");
        let update4 = phext::replace(update3.as_str(), coord4, "a bridge just close enough");
        assert_eq!(update4, "aaa\x17222\x183-\x19delta\x1Aa bridge just close enough\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'fff' with 'nifty'
        let coord5 = phext::to_coordinate("1.1.1/1.2.1/1.1.1");
        let update5 = phext::replace(update4.as_str(), coord5, "nifty");
        assert_eq!(update5, "aaa\x17222\x183-\x19delta\x1Aa bridge just close enough\x1Cnifty\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'ggg' with 'G8'
        let coord6 = phext::to_coordinate("1.1.1/2.1.1/1.1.1");
        let update6 = phext::replace(update5.as_str(), coord6, "G8");
        assert_eq!(update6, "aaa\x17222\x183-\x19delta\x1Aa bridge just close enough\x1Cnifty\x1DG8\x1Ehhh\x1Fiii\x01jjj");

        // replace 'hhh' with 'Hello World'
        let coord7 = phext::to_coordinate("1.1.2/1.1.1/1.1.1");
        let update7 = phext::replace(update6.as_str(), coord7, "Hello World");
        assert_eq!(update7, "aaa\x17222\x183-\x19delta\x1Aa bridge just close enough\x1Cnifty\x1DG8\x1EHello World\x1Fiii\x01jjj");

        // replace 'iii' with '_o_'
        let coord8 = phext::to_coordinate("1.2.1/1.1.1/1.1.1");
        let update8 = phext::replace(update7.as_str(), coord8, "_o_");
        assert_eq!(update8, "aaa\x17222\x183-\x19delta\x1Aa bridge just close enough\x1Cnifty\x1DG8\x1EHello World\x1F_o_\x01jjj");

        // replace 'jjj' with '/win'
        let coord9: phext::Coordinate = phext::to_coordinate("2.1.1/1.1.1/1.1.1");
        let update9 = phext::replace(update8.as_str(), coord9, "/win");
        assert_eq!(update9, "aaa\x17222\x183-\x19delta\x1Aa bridge just close enough\x1Cnifty\x1DG8\x1EHello World\x1F_o_\x01/win");

        // the api editor has trouble with this input...
        let coord_r0a = phext::to_coordinate("2.1.1/1.1.1/1.1.5");
        let update_r0a = phext::replace("hello world\x17scroll two", coord_r0a, "2.1.1-1.1.1-1.1.5");
        assert_eq!(update_r0a, "hello world\x17scroll two\x01\x17\x17\x17\x172.1.1-1.1.1-1.1.5");

        // regression from api testing
        // unit tests don't hit the failure I'm seeing through rocket...hmm - seems to be related to using library breaks
        let coord_r1a = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let update_r1a = phext::replace("", coord_r1a, "aaa");
        assert_eq!(update_r1a, "aaa");

        let coord_r1b = phext::to_coordinate("1.1.1/1.1.1/1.1.2");
        let update_r1b = phext::replace(update_r1a.as_str(), coord_r1b, "bbb");
        assert_eq!(update_r1b, "aaa\x17bbb");

        let coord_r1c = phext::to_coordinate("1.2.3/4.5.6/7.8.9");
        let update_r1c = phext::replace(update_r1b.as_str(), coord_r1c, "ccc");
        assert_eq!(update_r1c, "aaa\x17bbb\x1F\x1E\x1E\x1D\x1D\x1D\x1C\x1C\x1C\x1C\x1A\x1A\x1A\x1A\x1A\x19\x19\x19\x19\x19\x19\x18\x18\x18\x18\x18\x18\x18\x17\x17\x17\x17\x17\x17\x17\x17ccc");

        let coord_r1d = phext::to_coordinate("1.4.4/2.8.8/4.16.16");
        let update_r1d = phext::replace(update_r1c.as_str(), coord_r1d, "ddd");
        assert_eq!(update_r1d, "aaa\x17bbb\x1F\x1E\x1E\x1D\x1D\x1D\x1C\x1C\x1C\x1C\x1A\x1A\x1A\x1A\x1A\x19\x19\x19\x19\x19\x19\x18\x18\x18\x18\x18\x18\x18\x17\x17\x17\x17\x17\x17\x17\x17ccc\x1F\x1F\x1E\x1E\x1E\x1D\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x19\x19\x19\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17ddd");

        let coord_regression_1 = phext::to_coordinate("11.12.13/14.15.16/17.18.19");
        let update_regression_1 = phext::replace(update_r1d.as_str(), coord_regression_1, "eee");
        assert_eq!(update_regression_1, "aaa\x17bbb\x1F\x1E\x1E\x1D\x1D\x1D\x1C\x1C\x1C\x1C\x1A\x1A\x1A\x1A\x1A\x19\x19\x19\x19\x19\x19\x18\x18\x18\x18\x18\x18\x18\x17\x17\x17\x17\x17\x17\x17\x17ccc\x1F\x1F\x1E\x1E\x1E\x1D\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x19\x19\x19\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17ddd".to_owned() +
        "\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01" +
        "\x1F\x1F\x1F\x1F\x1F\x1F\x1F\x1F\x1F\x1F\x1F" +
        "\x1E\x1E\x1E\x1E\x1E\x1E\x1E\x1E\x1E\x1E\x1E\x1E" +
        "\x1D\x1D\x1D\x1D\x1D\x1D\x1D\x1D\x1D\x1D\x1D\x1D\x1D" +
        "\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1C\x1C" +
        "\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A\x1A" +
        "\x19\x19\x19\x19\x19\x19\x19\x19\x19\x19\x19\x19\x19\x19\x19\x19" +
        "\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18\x18" +
        "\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17\x17" +
        "eee");

        // finally found the bugger!
        let coord_regression_2 = phext::to_coordinate("1.1.1/1.1.2/1.1.2");
        let regression_2_baseline = "1.1.11.1.21.1.31.1.41.2.11.2.21.2.31.2.4".to_string() +
            "2.1.13.1.14.1.12/1.1.12/1.1.32.1/1.1.12.1.1/1.1.12/1.1.1/1.1.12.1/1.1.1/1.1.12.1.1/1.1.1/1.1.1";
        let update_regression_2 = phext::replace(regression_2_baseline.as_str(), coord_regression_2, "new content");
        assert_eq!(update_regression_2, "1.1.1\x171.1.2\x171.1.3\x171.1.4\x181.2.1\x171.2.2\x171.2.3\x171.2.4\x192.1.1\x193.1.1\x194.1.1\x1a2/1.1.1\x17new content\x172/1.1.3\x1c2.1/1.1.1\x1d2.1.1/1.1.1\x1e2/1.1.1/1.1.1\x1f2.1/1.1.1/1.1.1\x012.1.1/1.1.1/1.1.1");
    }

    #[test]
    fn test_coordinate_based_remove() {
        // replace 'aaa' with ''
        let coord1 = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let update1 = phext::remove("aaa\x17bbb\x18ccc\x19ddd\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj", coord1);
        assert_eq!(update1, "\x17bbb\x18ccc\x19ddd\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'bbb' with ''
        let coord2 = phext::to_coordinate("1.1.1/1.1.1/1.1.2");
        let update2 = phext::remove(update1.as_str(), coord2);
        assert_eq!(update2, "\x18ccc\x19ddd\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'ccc' with ''
        let coord3 = phext::to_coordinate("1.1.1/1.1.1/1.2.1");
        let update3 = phext::remove(update2.as_str(), coord3);
        assert_eq!(update3, "\x19ddd\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'ddd' with ''
        let coord4 = phext::to_coordinate("1.1.1/1.1.1/2.1.1");
        let update4 = phext::remove(update3.as_str(), coord4);
        assert_eq!(update4, "\x1Aeee\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'eee' with ''
        let coord5 = phext::to_coordinate("1.1.1/1.1.2/1.1.1");
        let update5 = phext::remove(update4.as_str(), coord5);
        assert_eq!(update5, "\x1Cfff\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'fff' with ''
        let coord6 = phext::to_coordinate("1.1.1/1.2.1/1.1.1");
        let update6 = phext::remove(update5.as_str(), coord6);
        assert_eq!(update6, "\x1Dggg\x1Ehhh\x1Fiii\x01jjj");

        // replace 'ggg' with ''
        let coord7 = phext::to_coordinate("1.1.1/2.1.1/1.1.1");
        let update7 = phext::remove(update6.as_str(), coord7);
        assert_eq!(update7, "\x1Ehhh\x1Fiii\x01jjj");

        // replace 'hhh' with ''
        let coord8 = phext::to_coordinate("1.1.2/1.1.1/1.1.1");
        let update8 = phext::remove(update7.as_str(), coord8);
        assert_eq!(update8, "\x1Fiii\x01jjj");

        // replace 'iii' with ''
        let coord9 = phext::to_coordinate("1.2.1/1.1.1/1.1.1");
        let update9 = phext::remove(update8.as_str(), coord9);
        assert_eq!(update9, "\x01jjj");

        // replace 'jjj' with ''
        let coord10 = phext::to_coordinate("2.1.1/1.1.1/1.1.1");
        let update10 = phext::remove(update9.as_str(), coord10);
        assert_eq!(update10, "");
    }

    #[test]
    fn test_range_based_replace() {
        let doc1 = "Before\x19text to be replaced\x1Calso this\x1Dand this\x17After";
        let range1 = phext::Range { start: phext::to_coordinate("1.1.1/1.1.1/2.1.1"),
                            end: phext::to_coordinate("1.1.1/2.1.1/1.1.1") };
        let update1 = phext::range_replace(doc1, range1, "");
        assert_eq!(update1, "Before\x19\x17After");

        let doc2 = "Before\x01Library two\x01Library three\x01Library four";
        let range2 = phext::Range { start: phext::to_coordinate("2.1.1/1.1.1/1.1.1"),
                            end: phext::to_coordinate("3.1.1/1.1.1/1.1.1") };

        let update2 = phext::range_replace(doc2, range2, "");
        assert_eq!(update2, "Before\x01\x01Library four");
    }

    #[test]
    fn test_next_scroll() {
        let doc1 = "3A\x17B2\x18C1";
        let (update1, next_start, remaining) = phext::next_scroll(doc1, phext::to_coordinate("1.1.1/1.1.1/1.1.1"));
        assert_eq!(update1.coord.to_string(), "1.1.1/1.1.1/1.1.1");
        assert_eq!(update1.scroll, "3A");
        assert_eq!(next_start.to_string(), "1.1.1/1.1.1/1.1.2");
        assert_eq!(remaining, "B2\x18C1");
    }

    #[test]
    fn test_phokenize() {
        let doc1 = "one\x17two\x17three\x17four";
        let mut expected1: Vec<phext::PositionedScroll> = Vec::new();
        expected1.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/1.1.1"), scroll: "one".to_string()});
        expected1.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/1.1.2"), scroll: "two".to_string()});
        expected1.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/1.1.3"), scroll: "three".to_string()});
        expected1.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/1.1.4"), scroll: "four".to_string()});
        let update1: Vec<PositionedScroll> = phext::phokenize(doc1);
        assert_eq!(update1, expected1);

        let doc2 = "one\x01two\x1Fthree\x1Efour\x1Dfive\x1Csix\x1Aseven\x19eight\x18nine\x17ten";
        let mut expected2: Vec<phext::PositionedScroll> = Vec::new();
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/1.1.1"), scroll: "one".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.1.1/1.1.1/1.1.1"), scroll: "two".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.2.1/1.1.1/1.1.1"), scroll: "three".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.2.2/1.1.1/1.1.1"), scroll: "four".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.2.2/2.1.1/1.1.1"), scroll: "five".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.2.2/2.2.1/1.1.1"), scroll: "six".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.2.2/2.2.2/1.1.1"), scroll: "seven".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.2.2/2.2.2/2.1.1"), scroll: "eight".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.2.2/2.2.2/2.2.1"), scroll: "nine".to_string()});
        expected2.push(PositionedScroll{ coord: phext::to_coordinate("2.2.2/2.2.2/2.2.2"), scroll: "ten".to_string()});
        let update2: Vec<PositionedScroll> = phext::phokenize(doc2);
        assert_eq!(update2, expected2);

        let doc3 = "one\x17two\x18three\x19four\x1afive\x1csix\x1dseven\x1eeight\x1fnine\x01ten";
        let mut expected3: Vec<phext::PositionedScroll> = Vec::new();
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/1.1.1"), scroll: "one".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/1.1.2"), scroll: "two".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/1.2.1"), scroll: "three".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.1/2.1.1"), scroll: "four".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.1.2/1.1.1"), scroll: "five".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/1.2.1/1.1.1"), scroll: "six".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.1.1/2.1.1/1.1.1"), scroll: "seven".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.1.2/1.1.1/1.1.1"), scroll: "eight".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("1.2.1/1.1.1/1.1.1"), scroll: "nine".to_string()});
        expected3.push(PositionedScroll{ coord: phext::to_coordinate("2.1.1/1.1.1/1.1.1"), scroll: "ten".to_string()});
        let update3: Vec<PositionedScroll> = phext::phokenize(doc3);
        assert_eq!(update3, expected3);

        let doc4 = "\x1A\x1C\x1D\x1E\x1F\x01stuff here";
        let mut expected4: Vec<phext::PositionedScroll> = Vec::new();
        expected4.push(PositionedScroll{ coord: phext::to_coordinate("2.1.1/1.1.1/1.1.1"), scroll: "stuff here".to_string()});
        let update4: Vec<PositionedScroll> = phext::phokenize(doc4);
        assert_eq!(update4, expected4);
    }

    #[test]
    fn test_last_empty_scroll() {

        // a regression discovered from SQ - see https://github.com/wbic16/SQ
        let doc1 = "hello\x17world\x17";
        let target1 = phext::to_coordinate("1.1.1/1.1.1/1.1.2");
        let bytes1: &[u8] = doc1.as_bytes();
        let parts1 = phext::get_subspace_coordinates(bytes1, target1);
        assert_eq!(parts1.0, 6);
        assert_eq!(parts1.1, 11);
        assert_eq!(parts1.2, target1);
        
        let test1 = phext::fetch(doc1, target1);
        assert_eq!(test1, "world");
    }

    #[test]
    fn test_merge() {
        let doc_1a = "3A\x17B2";
        let doc_1b = "4C\x17D1";
        let update_1 = phext::merge(doc_1a, doc_1b);
        assert_eq!(update_1, "3A4C\x17B2D1");

        let doc_2a = "Hello \x17I've come to talk";
        let doc_2b = "Darkness, my old friend.\x17 with you again.";
        let update_2 = phext::merge(doc_2a, doc_2b);
        assert_eq!(update_2, "Hello Darkness, my old friend.\x17I've come to talk with you again.");

        let doc_3a = "One\x17Two\x18Three\x19Four";
        let doc_3b = "1\x172\x183\x194";
        let update_3 = phext::merge(doc_3a, doc_3b);
        assert_eq!(update_3, "One1\x17Two2\x18Three3\x19Four4");

        let doc_4a = "\x1A\x1C\x1D\x1E\x1F\x01stuff here";
        let doc_4b = "\x1A\x1C\x1D\x1Eprecursor here\x1F\x01and more";
        let update_4 = phext::merge(doc_4a, doc_4b);
        assert_eq!(update_4, "\x1Eprecursor here\x01stuff hereand more");

        let doc_5a = "\x01\x01 Library at 3.1.1/1.1.1/1.1.1 \x1F Shelf at 3.2.1/1.1.1/1.1.1";
        let doc_5b = "\x01\x01\x01 Library 4.1.1/1.1.1/1.1.1 \x1E Series at 4.1.2/1.1.1/1.1.1";
        let update_5 = phext::merge(doc_5a, doc_5b);
        assert_eq!(update_5, "\x01\x01 Library at 3.1.1/1.1.1/1.1.1 \x1F Shelf at 3.2.1/1.1.1/1.1.1\x01 Library 4.1.1/1.1.1/1.1.1 \x1E Series at 4.1.2/1.1.1/1.1.1");

        let doc_6a = "\x1D Collection at 1.1.1/2.1.1/1.1.1\x1C Volume at 1.1.1/2.2.1/1.1.1";
        let doc_6b = "\x1D\x1D Collection at 1.1.1/3.1.1/1.1.1\x1C Volume at 1.1.1/3.2.1/1.1.1";
        let update_6 = phext::merge(doc_6a, doc_6b);
        assert_eq!(update_6, "\x1D Collection at 1.1.1/2.1.1/1.1.1\x1C Volume at 1.1.1/2.2.1/1.1.1\x1D Collection at 1.1.1/3.1.1/1.1.1\x1C Volume at 1.1.1/3.2.1/1.1.1");

        let doc_7a = "\x1ABook #2 Part 1\x1ABook #3 Part 1";
        let doc_7b = "\x1A + Part II\x1A + Part Deux";
        let update_7 = phext::merge(doc_7a, doc_7b);
        assert_eq!(update_7, "\x1ABook #2 Part 1 + Part II\x1ABook #3 Part 1 + Part Deux");

        let doc8a = "AA\x01BB\x01CC";
        let doc8b = "__\x01__\x01__";
        let update8 = phext::merge(doc8a, doc8b);
        assert_eq!(update8, "AA__\x01BB__\x01CC__");
    }

    #[test]
    fn test_subtract() {
        let doc1a = "Here's scroll one.\x17Scroll two.";
        let doc1b = "Just content at the first scroll";
        let update1 = phext::subtract(doc1a, doc1b);
        assert_eq!(update1, "\x17Scroll two.");
    }

    #[test]
    fn test_normalize() {
        let doc1 = "\x17Scroll two\x18\x18\x18\x18";
        let update1 = phext::normalize(doc1);
        assert_eq!(update1, "\x17Scroll two");
    }

    #[test]
    fn test_expand() {
        let doc1 = "nothing but line breaks\x0Ato test expansion to scrolls\x0Aline 3";
        let update1 = phext::expand(doc1);
        assert_eq!(update1, "nothing but line breaks\x17to test expansion to scrolls\x17line 3");

        let update2 = phext::expand(update1.as_str());
        assert_eq!(update2, "nothing but line breaks\x18to test expansion to scrolls\x18line 3");

        let update3 = phext::expand(update2.as_str());
        assert_eq!(update3, "nothing but line breaks\x19to test expansion to scrolls\x19line 3");

        let update4 = phext::expand(update3.as_str());
        assert_eq!(update4, "nothing but line breaks\x1Ato test expansion to scrolls\x1Aline 3");

        let update5 = phext::expand(update4.as_str());
        assert_eq!(update5, "nothing but line breaks\x1Cto test expansion to scrolls\x1Cline 3");

        let update6 = phext::expand(update5.as_str());
        assert_eq!(update6, "nothing but line breaks\x1Dto test expansion to scrolls\x1Dline 3");

        let update7 = phext::expand(update6.as_str());
        assert_eq!(update7, "nothing but line breaks\x1Eto test expansion to scrolls\x1Eline 3");

        let update8 = phext::expand(update7.as_str());
        assert_eq!(update8, "nothing but line breaks\x1Fto test expansion to scrolls\x1Fline 3");

        let update9 = phext::expand(update8.as_str());
        assert_eq!(update9, "nothing but line breaks\x01to test expansion to scrolls\x01line 3");

        let update10 = phext::expand(update9.as_str());
        assert_eq!(update10, "nothing but line breaks\x01to test expansion to scrolls\x01line 3");
    }

    #[test]
    fn test_contract() {
        let doc1 = "A more complex example than expand\x01----\x1F++++\x1E____\x1Doooo\x1C====\x1Azzzz\x19gggg\x18....\x17qqqq";
        let update1 = phext::contract(doc1);
        assert_eq!(update1, "A more complex example than expand\x1F----\x1E++++\x1D____\x1Coooo\x1A====\x19zzzz\x18gggg\x17....\x0Aqqqq");

        let update2 = phext::contract(update1.as_str());
        assert_eq!(update2, "A more complex example than expand\x1E----\x1D++++\x1C____\x1Aoooo\x19====\x18zzzz\x17gggg\x0A....\x0Aqqqq");
    }

    #[test]
    fn test_fs_read_write() {
        let filename = "unit-test.phext";
        let file = std::fs::File::create(&filename);
        let required = "Unable to locate ".to_owned() + &filename;
        let initial = "a simple phext doc with three scrolls\x17we just want to verify\x17that all of our breaks are making it through rust's fs layer.\x18section 2\x19chapter 2\x1Abook 2\x1Cvolume 2\x1Dcollection 2\x1Eseries 2\x1Fshelf 2\x01library 2";
        let _result = file.expect(&required).write_all(initial.as_bytes());
        let prior = std::fs::read_to_string(filename).expect("Unable to open phext");

        assert_eq!(prior, initial);
        let coordinate = "2.1.1/1.1.1/1.1.1";
        let message = phext::replace(prior.as_str(), phext::to_coordinate(coordinate), "still lib 2");
        assert_eq!(message, "a simple phext doc with three scrolls\x17we just want to verify\x17that all of our breaks are making it through rust's fs layer.\x18section 2\x19chapter 2\x1Abook 2\x1Cvolume 2\x1Dcollection 2\x1Eseries 2\x1Fshelf 2\x01still lib 2");
    }

    #[test]
    fn test_replace_create() {
        let initial = "A\x17B\x17C\x18D\x19E\x1AF\x1CG\x1DH\x1EI\x1FJ\x01K";
        let coordinate = "3.1.1/1.1.1/1.1.1";
        let message = phext::replace(initial, phext::to_coordinate(coordinate), "L");
        assert_eq!(message, "A\x17B\x17C\x18D\x19E\x1AF\x1CG\x1DH\x1EI\x1FJ\x01K\x01L");
    }

    #[test]
    fn test_summary() {
        let doc1 = "A short phext\nSecond line\x17second scroll.............................";
        let update1 = phext::create_summary(doc1);
        assert_eq!(update1, "A short phext...");

        let doc2 = "very terse";
        let update2 = phext::create_summary(doc2);
        assert_eq!(update2, "very terse");
    }

    #[test]
    fn test_navmap() {
        let example = "Just a couple of scrolls.\x17Second scroll\x17Third scroll";
        let result = phext::navmap("http://127.0.0.1/api/v1/index/", example);
        assert_eq!(result, "<ul>\n<li><a href=\"http://127.0.0.1/api/v1/index/1.1.1;1.1.1;1.1.1\">1.1.1/1.1.1/1.1.1 Just a couple of scrolls.</a></li>\n<li><a href=\"http://127.0.0.1/api/v1/index/1.1.1;1.1.1;1.1.2\">1.1.1/1.1.1/1.1.2 Second scroll</a></li>\n<li><a href=\"http://127.0.0.1/api/v1/index/1.1.1;1.1.1;1.1.3\">1.1.1/1.1.1/1.1.3 Third scroll</a></li>\n</ul>\n");
    }

    #[test]
    fn test_textmap() {
        let example = "Just a couple of scrolls.\x17Second scroll\x17Third scroll";
        let result = phext::textmap(example);
        assert_eq!(result, "* 1.1.1/1.1.1/1.1.1: Just a couple of scrolls.\n* 1.1.1/1.1.1/1.1.2: Second scroll\n* 1.1.1/1.1.1/1.1.3: Third scroll\n");
    }

    #[test]
    fn test_phext_index() {
        let example = "first scroll\x17second scroll\x18second section\x19second chapter\x1Abook 2\x1Cvolume 2\x1Dcollection 2\x1Eseries 2\x1Fshelf 2\x01library 2";
        let result = phext::index(example);
        assert_eq!(result, "0\x1713\x1827\x1942\x1a57\x1c64\x1d73\x1e86\x1f95\x01103");

        let coord1 = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let test1 = phext::offset(example, coord1);
        assert_eq!(test1, 0);

        let coord2 = phext::to_coordinate("1.1.1/1.1.1/1.1.2");
        let test2 = phext::offset(example, coord2);
        assert_eq!(test2, 13);

        let coord3 = phext::to_coordinate("1.1.1/1.1.1/1.2.1");
        let test3 = phext::offset(example, coord3);
        assert_eq!(test3, 27);

        let coord4 = phext::to_coordinate("1.1.1/1.1.1/2.1.1");
        let test4 = phext::offset(example, coord4);
        assert_eq!(test4, 42);

        let coord5 = phext::to_coordinate("1.1.1/1.1.2/1.1.1");
        let test5 = phext::offset(example, coord5);
        assert_eq!(test5, 57);

        let coord6 = phext::to_coordinate("1.1.1/1.2.1/1.1.1");
        let test6 = phext::offset(example, coord6);
        assert_eq!(test6, 64);

        let coord7 = phext::to_coordinate("1.1.1/2.1.1/1.1.1");
        let test7 = phext::offset(example, coord7);
        assert_eq!(test7, 73);

        let coord8 = phext::to_coordinate("1.1.2/1.1.1/1.1.1");
        let test8 = phext::offset(example, coord8);
        assert_eq!(test8, 86);

        let coord9 = phext::to_coordinate("1.2.1/1.1.1/1.1.1");
        let test9 = phext::offset(example, coord9);
        assert_eq!(test9, 95);

        let coord9 = phext::to_coordinate("2.1.1/1.1.1/1.1.1");
        let test9 = phext::offset(example, coord9);
        assert_eq!(test9, 103);

        let coord_invalid = phext::to_coordinate("2.1.1/1.1.1/1.2.1");
        let test_invalid = phext::offset(example, coord_invalid);
        assert_eq!(test_invalid, 103);

        assert_eq!(example.len(), 112);
    }

    #[test]
    fn test_scroll_manifest() {
        let example = "first scroll\x17second scroll\x18second section\x19second chapter\x1Abook 2\x1Cvolume 2\x1Dcollection 2\x1Eseries 2\x1Fshelf 2\x01library 2";
        let result = phext::manifest(example);

        let scroll0 = "00000000000000000000";
        let hash0 = phext::checksum(scroll0);
        assert_eq!(hash0, "7e79edd92a62a048e1cd24ffab542e34");

        let scroll1 = "first scroll";
        let hash1 = phext::checksum(scroll1);
        assert_eq!(hash1, "ba9d944e4967e29d48bae69ac2999699");

        let scroll2 = "second scroll";
        let hash2 = phext::checksum(scroll2);
        assert_eq!(hash2, "2fe1b2040314ac66f132dd3b4926157c");

        let scroll3 = "second section";
        let hash3 = phext::checksum(scroll3);
        assert_eq!(hash3, "fddb6916753b6f4e0b5281469134778b");

        let scroll4 = "second chapter";
        let hash4 = phext::checksum(scroll4);
        assert_eq!(hash4, "16ab5b1a0a997db95ec215a3bf2c57b3");

        let scroll5 = "book 2";
        let hash5 = phext::checksum(scroll5);
        assert_eq!(hash5, "0f20f79bf36f63e8fba25cc6765e2d0d");

        let scroll6 = "volume 2";
        let hash6 = phext::checksum(scroll6);
        assert_eq!(hash6, "7ead0c6fef43adb446fe3bda6fb0adc7");

        let scroll7 = "collection 2";
        let hash7 = phext::checksum(scroll7);
        assert_eq!(hash7, "78c12298931c6edede92962137a9280a");

        let scroll8 = "series 2";
        let hash8 = phext::checksum(scroll8);
        assert_eq!(hash8, "0f35100c84df601a490b7b63d7e8c0a8");

        let scroll9 = "shelf 2";
        let hash9 = phext::checksum(scroll9);
        assert_eq!(hash9, "3bbf7e67cb33d613a906bc5a3cbefd95");

        let scroll10 = "library 2";
        let hash10 = phext::checksum(scroll10);
        assert_eq!(hash10, "2e7fdd387196a8a2706ccb9ad6792bc3");

        let expected = format!("{}\x17{}\x18{}\x19{}\x1A{}\x1C{}\x1D{}\x1E{}\x1F{}\x01{}", hash1, hash2, hash3, hash4, hash5, hash6, hash7, hash8, hash9, hash10);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_phext_soundex_v1() {
        let sample = "it was the best of scrolls\x17it was the worst of scrolls\x17aaa\x17bbb\x17ccc\x17ddd\x17eee\x17fff\x17ggg\x17hhh\x17iii\x17jjj\x17kkk\x17lll\x18mmm\x18nnn\x18ooo\x18ppp\x19qqq\x19rrr\x19sss\x19ttt\x1auuu\x1avvv\x1awww\x1axxx\x1ayyy\x1azzz";
        let result = phext::soundex_v1(sample);
        assert_eq!(result, "36\x1741\x171\x174\x177\x1710\x171\x174\x177\x171\x171\x177\x177\x1713\x1816\x1816\x181\x184\x197\x1919\x197\x1910\x1a1\x1a4\x1a1\x1a7\x1a1\x1a7");
    }

    #[test]
    fn test_insert_performance_2k_scrolls() {
        let doc1 = "the quick brown fox jumped over the lazy dog";
        let mut x = 0;
        let mut next = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let mut result = vec!["".to_string()];

        let start = SystemTime::now();
        loop {
            x += 1;
            if x > 2000 {
                break;
            }
            if next.x.scroll > 32 {
                next.section_break();
            }
            if next.x.section > 32 {
                next.chapter_break();
            }
            if next.x.chapter > 32 {
                next.book_break();
            }
            result.push(phext::insert(result[x-1].clone(), next, doc1));
            next.scroll_break();
        }

        let end = SystemTime::now().duration_since(start).expect("get millis error");

        println!("Performance test took: {} ms", end.as_millis());
        let success = end.as_millis() < 5000;
        assert_eq!(success, true);

        // TODO: double-check this math
        let expected = phext::to_coordinate("1.1.1/1.1.1/2.31.17");
        assert_eq!(next, expected);

        let expected_doc1_length = 44;
        assert_eq!(doc1.len(), expected_doc1_length);

        // 2000 scrolls should be separated by 1999 delimiters
        let mut phext_tokens = 0;
        let mut scroll_breaks = 0;
        let mut section_breaks = 0;
        let mut chapter_breaks = 0;
        let check = result.last().expect("at least one").as_bytes();
        for byte in check {
            if phext::is_phext_break(*byte) {
                phext_tokens += 1;
            }
            if *byte == phext::SCROLL_BREAK as u8 {
                scroll_breaks += 1;
            }
            if *byte == phext::SECTION_BREAK as u8 {
                section_breaks += 1;
            }
            if *byte == phext::CHAPTER_BREAK as u8 {
                chapter_breaks += 1;
            }
        }
        let expected_tokens = 1999;
        assert_eq!(phext_tokens, expected_tokens);

        assert_eq!(scroll_breaks, 1937); // 1999 dimension breaks minus section and chapter breaks
        assert_eq!(section_breaks, 61);  // 63 sections with 61 delimiters (due to 1 chapter break)
        assert_eq!(chapter_breaks, 1);   // 2 chapters with 1 delimiter

        // doc1 * 1000 + delimiter count
        let expected_length = 2000 * expected_doc1_length + expected_tokens;
        assert_eq!(check.len(), expected_length);

        // note: raw performance is slow due to lack of optimization so far
        // for 2,000 scrolls on my laptop, we're averaging 2.3 ms per record

    }

    #[test]
    fn test_insert_performance_medium_scrolls() {
        let doc_template = "the quick brown fox jumped over the lazy dog\n";
        let mut doc1 = "".to_string();
        let mut x = 0;
        while x < 1000 {
            x += 1;
            doc1.push_str(doc_template);
        }
        let doc1 = doc1.as_str();
        let mut next = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let mut result = vec!["".to_string()];

        let start = SystemTime::now();
        x = 0;
        loop {
            x += 1;
            if x > 25 {
                break;
            }
            if next.x.scroll > 5 {
                next.section_break();
            }
            if next.x.section > 5 {
                next.chapter_break();
            }
            if next.x.chapter > 5 {
                next.book_break();
            }
            result.push(phext::insert(result[x-1].clone(), next, doc1));
            next.scroll_break();
        }

        let end = SystemTime::now().duration_since(start).expect("get millis error");

        println!("Performance test took: {} ms", end.as_millis());
        let success = end.as_millis() < 1000;
        assert_eq!(success, true);

        // TODO: double-check this math
        let expected = phext::to_coordinate("1.1.1/1.1.1/1.5.6");
        assert_eq!(next, expected);

        let expected_doc1_length = 45000; // counting line breaks
        assert_eq!(doc1.len(), expected_doc1_length);

        // 2000 scrolls should be separated by 1999 delimiters
        let mut phext_tokens = 0;
        let mut line_breaks = 0;
        let mut scroll_breaks = 0;
        let mut section_breaks = 0;
        let mut chapter_breaks = 0;
        let check = result.last().expect("at least one").as_bytes();
        for byte in check {
            if phext::is_phext_break(*byte) {
                phext_tokens += 1;
            }
            if *byte == phext::LINE_BREAK as u8 {
                line_breaks += 1;
            }
            if *byte == phext::SCROLL_BREAK as u8 {
                scroll_breaks += 1;
            }
            if *byte == phext::SECTION_BREAK as u8 {
                section_breaks += 1;
            }
            if *byte == phext::CHAPTER_BREAK as u8 {
                chapter_breaks += 1;
            }
        }
        let expected_tokens = 25024;
        assert_eq!(phext_tokens, expected_tokens);

        assert_eq!(line_breaks, 25000); //
        assert_eq!(scroll_breaks, 20);  //
        assert_eq!(section_breaks, 4);  //
        assert_eq!(chapter_breaks, 0);  //

        // doc1 * 1000 + delimiter count
        let expected_length = 25 * (expected_doc1_length-1000) + expected_tokens;
        assert_eq!(check.len(), expected_length);

    }

    #[test]
    fn test_new_operators() {
        let ps = phext::PositionedScroll::new();
        assert_eq!(ps.scroll, "".to_string());
        assert_eq!(ps.coord, phext::to_coordinate("1.1.1/1.1.1/1.1.1"));

        let coord = phext::Coordinate::new();
        assert_eq!(coord.z, phext::ZCoordinate::new());
        assert_eq!(coord.y, phext::YCoordinate::new());
        assert_eq!(coord.x, phext::XCoordinate::new());

        let range = phext::Range::new();
        assert_eq!(range.start, phext::Coordinate::new());
        assert_eq!(range.end, phext::Coordinate::new());

        let zc = phext::ZCoordinate::new();
        assert_eq!(zc.library, 1);
        assert_eq!(zc.shelf, 1);
        assert_eq!(zc.series, 1);

        let yc = phext::YCoordinate::new();
        assert_eq!(yc.collection, 1);
        assert_eq!(yc.volume, 1);
        assert_eq!(yc.book, 1);

        let xc = phext::XCoordinate::new();
        assert_eq!(xc.chapter, 1);
        assert_eq!(xc.section, 1);
        assert_eq!(xc.scroll, 1);
    }

    #[test]
    fn test_hash_support() {
        let mut stuff = phext::explode("hello world\x17\x17\x17scroll 4\x01Library 2");
        let scroll1_address = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
        let scroll2_address = phext::to_coordinate("1.1.1/1.1.1/1.1.4");
        let scroll3_address = phext::to_coordinate("2.1.1/1.1.1/1.1.1");
        assert_eq!(stuff[&scroll1_address], "hello world");
        assert_eq!(stuff[&scroll2_address], "scroll 4");
        assert_eq!(stuff[&scroll3_address], "Library 2");

        let scroll4_address = phext::to_coordinate("2.3.4/5.6.7/8.9.1");
        stuff.insert(scroll4_address, "random insertion".to_string());

        let serialized = phext::implode(stuff);
        assert_eq!(serialized, "hello world\x17\x17\x17scroll 4\x01Library 2\x1f\x1f\x1e\x1e\x1e\x1d\x1d\x1d\x1d\x1c\x1c\x1c\x1c\x1c\x1a\x1a\x1a\x1a\x1a\x1a\x19\x19\x19\x19\x19\x19\x19\x18\x18\x18\x18\x18\x18\x18\x18random insertion");
    }
}