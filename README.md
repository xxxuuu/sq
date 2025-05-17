# sq

sq queries anything with SQL directly in your terminal

## Usage Examples

```bash
$ sq --help
Query anything with SQL directly in your terminal

Usage: sq [--disable-type-infer] [<sql>]

Available options:
        --disable-type-infer  Disable type inference
    -h, --help                Prints help information
```

```bash
$ ps -ef | sq 'SELECT * FROM stdin LIMIT 5'
+------+-----+------+---+-------+-----+----------+--------------------------------------------------------------------+
| UID  | PID | PPID | C | STIME | TTY | TIME     | CMD                                                                |
+------+-----+------+---+-------+-----+----------+--------------------------------------------------------------------+
| root | 1   | 0    | 0 | Mar13 | ?   | 00:02:12 | /usr/lib/systemd/systemd --switched-root --system --deserialize 17 |
| root | 2   | 0    | 0 | Mar13 | ?   | 00:00:04 | [kthreadd]                                                         |
| root | 3   | 2    | 0 | Mar13 | ?   | 00:00:00 | [rcu_gp]                                                           |
| root | 4   | 2    | 0 | Mar13 | ?   | 00:00:00 | [rcu_par_gp]                                                       |
| root | 5   | 2    | 0 | Mar13 | ?   | 00:00:00 | [netns]                                                            |
+------+-----+------+---+-------+-----+----------+--------------------------------------------------------------------+

$ ps -ef | sq 'SELECT "UID", COUNT(*) "CNT" FROM stdin GROUP BY "UID" ORDER BY "CNT" DESC LIMIT 5'
+----------+-----+
| UID      | CNT |
+----------+-----+
| root     | 262 |
| systemd+ | 6   |
| td-agent | 2   |
| dbus     | 1   |
| polkitd  | 1   |
+----------+-----+
```

Type inference may be incorrect, and the table schema can be viewed by `DESCRIBE stdin`.
```bash
$ ps -ef | sq 'DESCRIBE stdin'
+-------------+-----------+-------------+
| column_name | data_type | is_nullable |
+-------------+-----------+-------------+
| UID         | Int64     | NO          |
| PID         | Int64     | NO          |
| PPID        | Int64     | NO          |
| C           | Int64     | NO          |
| STIME       | Utf8      | NO          |
| TTY         | Utf8      | NO          |
| TIME        | Utf8      | NO          |
| CMD         | Utf8      | NO          |
+-------------+-----------+-------------+
```

If this is not as expected, type inference can be disabled by using the `--disable-type-infer` flag.
```bash
$ ps -ef | sq --disable-type-infer 'DESCRIBE stdin'
+-------------+-----------+-------------+
| column_name | data_type | is_nullable |
+-------------+-----------+-------------+
| UID         | Utf8      | NO          |
| PID         | Utf8      | NO          |
| PPID        | Utf8      | NO          |
| C           | Utf8      | NO          |
| STIME       | Utf8      | NO          |
| TTY         | Utf8      | NO          |
| TIME        | Utf8      | NO          |
| CMD         | Utf8      | NO          |
+-------------+-----------+-------------+
```