# sq

sq queries anything with SQL directly in your terminal

## Usage Examples

```bash
$ ps -ef | sq 'SELECT * FROM stdin LIMIT 5'
+------+-----+------+---+-------+-----+----------+--------------------------+
| UID  | PID | PPID | C | STIME | TTY | TIME     | CMD                      |
+------+-----+------+---+-------+-----+----------+--------------------------+
| root | 1   | 0    | 0 | Mar13 | ?   | 00:02:01 | /usr/lib/systemd/systemd |
| root | 2   | 0    | 0 | Mar13 | ?   | 00:00:03 | [kthreadd]               |
| root | 3   | 2    | 0 | Mar13 | ?   | 00:00:00 | [rcu_gp]                 |
| root | 4   | 2    | 0 | Mar13 | ?   | 00:00:00 | [rcu_par_gp]             |
| root | 5   | 2    | 0 | Mar13 | ?   | 00:00:00 | [netns]                  |
+------+-----+------+---+-------+-----+----------+--------------------------+

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