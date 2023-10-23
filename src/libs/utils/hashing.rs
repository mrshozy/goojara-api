pub struct SimpleHash;

impl SimpleHash {
    pub fn hash(input: &str) -> String {
        let message = SimpleHash::pad_message(input);
        let k: [i64; 4] = [1518500249, 1859775393, 2400959708, 3395469782];
        let mut h_values = [1732584193, 4023233417, 2562383102, 271733878, 3285377520];
        let mut message_chars = message.chars().collect::<Vec<char>>();
        message_chars.push('\u{80}');
        let num_blocks = ((message_chars.len() as f64 / 4.0 + 2.0) / 16.0).ceil() as usize;
        let mut message_blocks = vec![vec![0u32; 16]; num_blocks];
        for block_num in 0..num_blocks {
            for word_num in 0..16 {
                let char_index = 64 * block_num + 4 * word_num;
                message_blocks[block_num][word_num] = ((if char_index < message_chars.len() {
                    message_chars[char_index]
                } else {
                    0 as char
                } as u32) << 24)
                    | ((if char_index + 1 < message_chars.len() {
                    message_chars[char_index + 1]
                } else {
                    0 as char
                } as u32) << 16)
                    | ((if char_index + 2 < message_chars.len() {
                    message_chars[char_index + 2]
                } else {
                    0 as char
                } as u32) << 8)
                    | (if char_index + 3 < message_chars.len() {
                    message_chars[char_index + 3]
                } else {
                    0 as char
                } as u32);
            }
        }

        message_blocks[num_blocks - 1][14] = (8.0 * (message_chars.len() - 1) as f64 / 2f64.powi(32)).floor() as u32;
        message_blocks[num_blocks - 1][15] = (8 * (message_chars.len() - 1) as u32) & 4294967295;

        for block_num in 0..num_blocks {
            let mut temp = vec![0u32; 80];
            for i in 0..16 {
                temp[i] = message_blocks[block_num][i];
            }

            for i in 16..80 {
                temp[i] = SimpleHash::sha1_func(temp[i - 3] ^ temp[i - 8] ^ temp[i - 14] ^ temp[i - 16], 1);
            }

            let (mut a, mut b, mut c, mut d, mut e) = (h_values[0], h_values[1], h_values[2], h_values[3], h_values[4]);

            for i in 0..80 {
                let t = (i as f64 / 20.0);
                let t = SimpleHash::sha1_func(a as u32, 5) as i64 + SimpleHash::func(t, b, c, d) as i64 + e + k[t as usize] as i64 + temp[i] as i64;
                e = d;
                d = c;
                c = SimpleHash::sha1_func(b as u32, 30) as i64;
                b = a;
                a = t;
            }

            h_values[0] = (h_values[0] + a) & 4294967295;
            h_values[1] = (h_values[1] + b) & 4294967295;
            h_values[2] = (h_values[2] + c) & 4294967295;
            h_values[3] = (h_values[3] + d) & 4294967295;
            h_values[4] = (h_values[4] + e) & 4294967295;
        }

        let result: Vec<String> = h_values.iter().map(|&x| format!("{:08x}", x)).collect();
        result.join("")
    }

    fn func(t: f64, b: i64, c: i64, d: i64) -> i32 {
        match t as i32 {
            0 => (b & c) as i32 ^ (!b & d) as i32,
            1 => (b ^ c ^ d) as i32,
            2 => (b & c) as i32 ^ (b & d) as i32 ^ (c & d) as i32,
            3 => (b ^ c ^ d) as i32,
            _ => 0,
        }
    }

    fn sha1_func(a: u32, e: u32) -> u32 {
        (a << e) | (a >> (32 - e))
    }

    fn pad_message(input: &str) -> String {
        let encoded = SimpleHash::encode(input);
        String::from_utf8(encoded).unwrap()
    }

    fn encode(input: &str) -> Vec<u8> {
        input
            .chars()
            .flat_map(|c| c.to_string().into_bytes())
            .collect()
    }
}