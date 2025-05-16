use std::{cell::OnceCell, cmp};

use unicode_segmentation::UnicodeSegmentation;

struct Column<'a> {
    name: &'a str,
    byte_range: std::ops::Range<usize>,
    position: std::ops::Range<usize>,
}

pub struct TableParser<'a> {
    header: OnceCell<Vec<Column<'a>>>,
}

impl<'a> TableParser<'a> {
    pub fn new() -> Self {
        Self {
            header: OnceCell::new(),
        }
    }

    fn map_header(&self) -> Vec<&'a str> {
        self.header.get().unwrap().iter().map(|c| c.name).collect()
    }

    fn parse_line_to_columns(&self, line: &'a str) -> Vec<Column<'a>> {
        let mut cols = vec![];
        let mut start: Option<(usize, usize)> = None;
        let chars = UnicodeSegmentation::grapheme_indices(line, true);
        let mut len = 0;
        for (pos, (idx, char)) in chars.into_iter().enumerate() {
            len = pos;
            if char.chars().all(|c| c.is_whitespace()) {
                if let Some((_start_pos, _start_idx)) = start {
                    cols.push(Column {
                        name: &line[_start_idx..idx],
                        byte_range: _start_idx..idx,
                        position: _start_pos..pos,
                    });
                    start = None;
                }
                continue;
            }
            if start.is_none() {
                start = Some((pos, idx));
            }
        }
        if let Some((_start_pos, _start_idx)) = start {
            cols.push(Column {
                name: &line[_start_idx..],
                byte_range: _start_idx..line.len(),
                position: _start_pos..len,
            });
        }
        cols
    }

    pub fn parse_header(&self, line: &'a str) -> Vec<&'a str> {
        self.header.get_or_init(|| self.parse_line_to_columns(line));
        self.map_header()
    }

    fn calc_distance(a: &std::ops::Range<usize>, b: &std::ops::Range<usize>) -> usize {
        cmp::min(
            (a.start as isize - b.start as isize).abs(),
            (a.end as isize - b.end as isize).abs(),
        ) as usize
    }

    pub fn parse_row(&self, line: &'a str) -> Vec<&'a str> {
        let Some(header) = self.header.get() else {
            return vec![];
        };

        let cols = self.parse_line_to_columns(line);

        let mut cols_by_header: Vec<Vec<Column>> = Vec::with_capacity(header.len());
        for _ in 0..header.len() {
            cols_by_header.push(vec![]);
        }
        for col in cols {
            // TODO: optimize this
            let mut closest_header = None;
            for (i, h) in header.iter().enumerate() {
                let Some(_closest_header) = closest_header else {
                    closest_header = Some((h, i, Self::calc_distance(&col.position, &h.position)));
                    continue;
                };
                let dis = Self::calc_distance(&col.position, &h.position);
                if dis < _closest_header.2 {
                    closest_header = Some((h, i, dis));
                }
            }
            if let Some((_, i, _)) = closest_header {
                cols_by_header[i].push(col);
            }
        }

        let mut res = Vec::with_capacity(header.len());
        for cols in cols_by_header.iter() {
            let (start, end) = (cols.first(), cols.last());
            let Some(start) = start else {
                res.push("");
                continue;
            };
            let Some(end) = end else {
                res.push("");
                continue;
            };
            res.push(&line[start.byte_range.start..end.byte_range.end]);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    fn case_ps() -> (&'static str, Vec<&'static str>, Vec<Vec<&'static str>>) {
        let input = "
UID          PID    PPID  C STIME TTY          TIME CMD
systemd+ 3777307 3777287  0 Apr22 ?        00:00:03 postgres -c config_file=/etc/postgresql/postgresql.conf
systemd+ 3777472 3777307  0 Apr22 ?        00:00:01 postgres: checkpointer
systemd+ 3777473 3777307  0 Apr22 ?        00:00:21 postgres: background writer";
        let expected_header = vec!["UID", "PID", "PPID", "C", "STIME", "TTY", "TIME", "CMD"];
        let expected_rows = vec![
            vec![
                "systemd+",
                "3777307",
                "3777287",
                "0",
                "Apr22",
                "?",
                "00:00:03",
                "postgres -c config_file=/etc/postgresql/postgresql.conf",
            ],
            vec![
                "systemd+",
                "3777472",
                "3777307",
                "0",
                "Apr22",
                "?",
                "00:00:01",
                "postgres: checkpointer",
            ],
            vec![
                "systemd+",
                "3777473",
                "3777307",
                "0",
                "Apr22",
                "?",
                "00:00:21",
                "postgres: background writer",
            ],
        ];
        (input, expected_header, expected_rows)
    }

    fn case_netstat() -> (&'static str, Vec<&'static str>, Vec<Vec<&'static str>>) {
        let input = "
Proto RefCnt Flags       Type       State         I-Node   Path
unix  2      [ ]         DGRAM                    65336200 /run/user/0/systemd/notify
unix  3      [ ]         DGRAM      CONNECTED     339      /run/systemd/notify
unix  2      [ ]         DGRAM                    340      /run/systemd/cgroups-agent
unix  12     [ ]         DGRAM      CONNECTED     350      /run/systemd/journal/dev-log
unix  7      [ ]         DGRAM      CONNECTED     354      /run/systemd/journal/socket
unix  2      [ ]         DGRAM                    41670369 @004fa";
        let expected_header = vec![
            "Proto", "RefCnt", "Flags", "Type", "State", "I-Node", "Path",
        ];
        let expected_rows = vec![
            vec![
                "unix",
                "2",
                "[ ]",
                "DGRAM",
                "",
                "65336200",
                "/run/user/0/systemd/notify",
            ],
            vec![
                "unix",
                "3",
                "[ ]",
                "DGRAM",
                "CONNECTED",
                "339",
                "/run/systemd/notify",
            ],
            vec![
                "unix",
                "2",
                "[ ]",
                "DGRAM",
                "",
                "340",
                "/run/systemd/cgroups-agent",
            ],
            vec![
                "unix",
                "12",
                "[ ]",
                "DGRAM",
                "CONNECTED",
                "350",
                "/run/systemd/journal/dev-log",
            ],
            vec![
                "unix",
                "7",
                "[ ]",
                "DGRAM",
                "CONNECTED",
                "354",
                "/run/systemd/journal/socket",
            ],
            vec!["unix", "2", "[ ]", "DGRAM", "", "41670369", "@004fa"],
        ];
        (input, expected_header, expected_rows)
    }

    fn case_lsblk() -> (&'static str, Vec<&'static str>, Vec<Vec<&'static str>>) {
        let input = "
NAME   MAJ:MIN RM  SIZE RO TYPE MOUNTPOINT
sr0     11:0    1  390K  0 rom  
vda    253:0    0  512G  0 disk 
├─vda1 253:1    0    1G  0 part /boot
└─vda2 253:2    0  511G  0 part /";
        let expected_header = vec!["NAME", "MAJ:MIN", "RM", "SIZE", "RO", "TYPE", "MOUNTPOINT"];
        let expected_rows = vec![
            vec!["sr0", "11:0", "1", "390K", "0", "rom", ""],
            vec!["vda", "253:0", "0", "512G", "0", "disk", ""],
            vec!["├─vda1", "253:1", "0", "1G", "0", "part", "/boot"],
            vec!["└─vda2", "253:2", "0", "511G", "0", "part", "/"],
        ];
        (input, expected_header, expected_rows)
    }

    #[test]
    fn test_parse() {
        let test_cases = vec![case_ps(), case_netstat(), case_lsblk()];

        for (input, expected_header, expected_rows) in test_cases {
            let line = input.lines().skip(1).collect::<Vec<_>>();
            let mut line = line.iter();
            let parser = super::TableParser::new();
            let header = parser.parse_header(line.next().unwrap());
            assert_eq!(header, expected_header);
            for expected_row in expected_rows {
                let parsed_row = parser.parse_row(line.next().unwrap());
                assert_eq!(parsed_row, *expected_row);
            }
        }
    }
}
