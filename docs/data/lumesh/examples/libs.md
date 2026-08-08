## Builtin Libs
USEAGE:
`<module-name>.<func-name>(arg1,arg2)`
`about.bin()`
`string.red('x')`
`string.red x`
NOTE:

- if arg is lambda expression, use square call method only.
- NEVER use lib name as var name.
- fuctions in top never need `top.` prefix, e.g. `cd -`

### Top level
assert <expr> [expr] [message]
cd [path=~]
cwd 
ddebug <args>...
debug <args>...
dig <map|list|set|range|table> <path>
eprint <args>...
eprintln <args>...
eval <expr>
eval_str <string>
exec <expr>
exec_str <string>
exit [status=0]
flatten <collection>
format <template> <args>...
get_env <var>
get_local <var>
get_var <var>
help [libs|tops|doc|<lib>|<lib>.<func>|<top_func>]
import <path>
include <path>
jobs [-k id]
len <list|set|map|table|range|string|bytes>
not <boolean>...
pprint <value>...
print <args>...
println <args>...
quote <expr>
read [-p prompt] [-n max_chars] [-s silent] [-t timeout_secs]
repeat <expr> <n>
rev <string|list|table|bytes>
select <table> <columns...>
set_root <var> <val>
symof <value>
tap <args>...
throw <msg>
typeof <value>
unset_root <var>
when <condition> <execute>
where <table> <condition>
### about
bin 
history 
info 
prelude 
version 
### boolean
and <boolean1>...
not <boolean1>...
or <boolean1>...
### bytes
concat <bytes> <bytes>
contains <bytes> <bytes>
ends_with <bytes> <bytes>
from <string>
from_base64 <base64_string>
from_escaped <string>
from_hex <hex_string>
from_list <list>
is_empty <bytes>
len <bytes>
pop <bytes>
position <bytes> <bytes>
push <bytes> <int>
repeat <bytes> <n>
reverse <bytes>
slice <bytes> <start> <end>
split <bytes> <int>
starts_with <bytes> <bytes>
to_base64 <bytes>
to_hex <bytes>
to_list <bytes>
to_string <bytes>
### console
alt_screen <bool>
bell 
clear 
cursor_down <n>
cursor_hide 
cursor_left <n>
cursor_restore 
cursor_right <n>
cursor_save 
cursor_show 
cursor_to <x> <y>
cursor_up <n>
discard <args>...
flush 
height 
keys 
line_wrap <bool>
print_tty <text>
raw_mode [bool]
read_key 
read_line [prompt]
read_password [prompt]
title <string>
width 
write <text> <x> <y>
### filesize
b <filesize>
from <size_str|byte_int>
gb <filesize>
kb <filesize>
mb <filesize>
tb <filesize>
to_string <filesize>
### from
cmd <output> [split_regex] [headers...]
csv <csv_string>
jq <json_string> <query_string>
json <json_string>
script <script_string>
toml <toml_string>
### fs
abs <path>
append <content> <file>
base_name <path>
canon <path>
chmod <path> <mode:octal>
chown <path> <uid> <gid>
cp <source> <destination>
dir_name <path>
exists <path>
extension <path>
glob <pattern>
head <file> [n=10]
is_dir <path>
is_file <path>
join <segment>...
ls [-l|a|h|t|L|c|u|m|p|?] [path]
mkdir <path>
mv <source> <destination>
parent <path>
read <file>
read_link <link_path>
rm <path>
rmdir <path>
stem <path>
symlink <source> <link_path>
tail <file> [n=10]
touch <path>
tree [depth=3] [path]
write [content] <file>
### hmap
contains_key <map> <key>
contains_value <map> <value>
difference <map1> <map2>
dig <map|list|range> <path>
filter <map> <fn>
find <map> <fn>
flatten <map>
from_list <list>
get <map> <key>
insert <map> <key> <value>
intersection <map1> <map2>
is_empty <map>
keys <map>
len <map>
map <map> <entry_fn>
merge <map1> <map2> [<map3>...]
remove <map> <key>
set <map> <key> <value>
to_bmap <map>
to_list <map>
union <map1> <map2>
values <map>
### into
boolean <value>
caesar <string> [shift=13]
csv <expr>
filesize <size_str|int>
float <str|num|bool>
highlight <script>
int <str|num|bool>
json <expr>
pretty <expr>
safe <str>
string <value>
strip <string>
table <output> [split_regex] [headers...]
time <str> [fmt]
toml <expr>
### list
all <list> <fn>
any <list> <fn>
average <num1> <num2>... | <array>
chunks <list> <size>
concat <list1|item1> <list2|item2>...
contains <list> <item>
dig <map|list|range> <path>
fill <value> <n>
filter <list> <fn>
filter_map <list> <fn>
find <list> <item|fn> [skip_n=0]
first <list> [n=1]
flatten <collection>
fold <list> <fn> [init=0]
from <range>
get <list> <index>
group <list> <key_fn|key>
insert <list> <index> <value>
is_empty <list>
items <list>
join <list> <separator>
last <list> [n=1]
len <list>
map <list> <fn>
max <num1> <num2>... | <array>
min <num1> <num2>... | <array>
position <list> <item|fn> [skip_n=0]
push <list> <element>
remove <list> <item> [all=false]
remove_at <list> <index> [count=1]
rev <list>
rfind <list> <item|fn> [skip_n=0]
rfold <list> <fn> [init=0]
rotate <list> <n>
rposition <list> <item|fn> [skip_n=0]
sample <list> <n>
set <list> <index> <value>
shuffle <list>
skip <list> <count>
slice <list> <start> <end>
sort <list> [key_fn|±key...]
splice <list> <start> <delete_count> [items...]
split_at <list> <index>
split_first <list>
sum <num1> <num2>... | <array>
swap <list> <i> <j>
take <list> <count>
to_hmap <list> [key_fn] [val_fn]
to_map <list> [key_fn] [val_fn]
to_set <list>
transpose <matrix>
unique <list>
unzip <list_of_pairs>
windows <list> <size>
zip <list1> <list2>
### log
debug <msg>
disable 
echo <msg>
enable [int]
error <msg>
info <msg>
is_enabled <level>
level [int]
trace <msg>
warn <msg>
### map
contains_key <map> <key>
contains_value <map> <value>
difference <map1> <map2>
dig <map|list|range> <path>
filter <map> <fn>
find <map> <fn>
first <map>
flatten <map>
from_list <list>
get <map> <key>
insert <map> <key> <value>
intersection <map1> <map2>
is_empty <map>
keys <map>
last <map>
len <map>
map <map> <map_fn>
merge <map1> <map2> [<map3>...]
remove <map> <key>
set <map> <key> <value>
to_hmap <map>
to_list <map>
union <map1> <map2>
values <map>
### math
abs <number>
acos <value>
acosh <value>
asin <value>
asinh <value>
atan <value>
atanh <value>
average <num1> <num2>... | <array>
bit_and <int1> <int2>
bit_not <integer>
bit_or <int1> <int2>
bit_shl <integer> <bits>
bit_shr <integer> <bits>
bit_xor <int1> <int2>
cbrt <number>
ceil <number>
clamp <value> <min> <max>
cos <radians>
cos_pi <x>
cosh <value>
eq <a> <b>
exp <x>
exp2 <x>
floor <number>
gcd <int1> <int2>
ge <a> <b>
gt <a> <b>
hypot <x> <y>
is_even <integer>
is_odd <integer>
lcm <int1> <int2>
le <a> <b>
ln <number>
log <number> <base>
log10 <number>
log2 <number>
lt <a> <b>
max <num1> <num2>... | <array>
min <num1> <num2>... | <array>
ne <a> <b>
pow <base> <exponent>
rem <a> <b>
round <number>
signum <number>
sin <radians>
sin_pi <x>
sinh <value>
sqrt <number>
sum <num1> <num2>... | <array>
tan <radians>
tan_pi <x>
tanh <value>
to_degrees <radians>
to_radians <degrees>
to_string <number>
trunc <number>
### rand
alpha [len=1]
alphanum [len=1]
chance [p=0.5]
choose <list>
float [min] [max]
int [min] [max]
ratio <num> <den>
sample <list> <n>
seed <integer>
shuffle <list>
### regex
capture <pattern> <text>
captures <pattern> <text>
find <pattern> <text>
find_all <pattern> <text>
from <pattern_string> [flag]
is_match <pattern> <text>
named_captures <pattern> <text>
replace <text> <pattern> <replacement>
replace_all <text> <pattern> <replacement>
split <pattern> <text>
### set
all <set> <fn>
any <set> <fn>
contains <set> <item>
difference <set1> <set2>
filter <set> <fn>
find <set> <fn>
first <set>
from_list <list>
get <set> <index>
insert <set> <item>
intersection <set1> <set2>
is_disjoint <set1> <set2>
is_empty <set>
is_subset <set1> <set2>
is_superset <set1> <set2>
last <set>
len <set>
map <set> <fn>
remove <set> <item>
split_first <set>
split_last <set>
symmetric_difference <set1> <set2>
to_list <set>
union <set1> <set2>
### string
black <string>
blink <string>
blue <string>
bold <string>
center <string> <length> [pad_char=' ']
chars <string>
clr <string> <0..256>
clr_bg <string> <0..256>
color <string> <#hex|name|r,g,b>
color_bg <string> <#hex|name|r,g,b>
colors [swatches?]
concat <string>...
contains <string> <substring>
cyan <string>
dim <string>
ends_with <string> <substring>
escape <string>
get <string> <index>
green <string>
grep <string> <substring>
href <url> <text>
insert <string> <index> <string>
invert <string>
is_alpha <string>
is_alphanumeric <string>
is_ascii <string>
is_ascii_control <string>
is_ascii_digit <string>
is_ascii_hexdigit <string>
is_ascii_punctuation <string>
is_empty <string>
is_lower <string>
is_numeric <string>
is_title <string>
is_upper <string>
is_whitespace <string>
italic <string>
len <string>
lines <string>
lower <string>
magenta <string>
max_len <string>
pad_end <string> <length> [pad_char=' ']
pad_start <string> <length> [pad_char=' ']
paragraphs <string>
position <string> <substring> [start]
red <string>
repeat <string> <count>
replace <string> <old> <new>
rev <string>
slice <string> <start> [end]
sort <string> ['+'|'-'|key_fn]
split <string> [delimiter]
split_at <string> <index>
starts_with <string> <substring>
strike <string>
strip_ansi <string>
strip_prefix <string> <prefix>
strip_suffix <string> <suffix>
title <string>
to_filesize <size_str>
to_float <value>
to_int <value>
to_safe <str>
to_table <output> [regex] [headers...]
to_time <str> [fmt]
trim <string>
trim_end <string>
trim_start <string>
underline <string>
unescape <string>
upper <string>
white <string>
words <string>
words_quoted <string>
wrap <string> <width>
yellow <string>
### sys
defined <var>
dirs 
error_codes
env [var]
has <var>
info 
max_runtime [depth]
max_syntax [depth]
max_usemode [depth]
modes 
set_cfm <boolean|none>
set_pdm <boolean>
set_strict <boolean>
vars 
### table
filter <table> <cell|fn>
first <table> [n=1]
first_map <table> [n=1]
from_maps <list_of_maps>
get <table> <index>
get_cell <table> <row_index> <header|index>
get_column <table> <header|index>
get_map <table> <index>
grep <table> <string>
header_len <table>
headers <table>
is_empty <table>
last <table> [n=1]
last_map <table> [n=1]
len <table>
position <table> <cell|fn> [start=0]
push <table> <list|set>
rows <table>
rows_map <table>
rposition <table> <cell|fn> [start=0]
select <table> <cols...>
slice <table> <start> <end>
sort <list> [key_fn|±key...]
to_csv <table>
### time
add [datetime] <duration>
day [datetime]
diff <datetime1> [datetime2] <unit>
display [datetime]
fmt [datetime] <format_string>
hour [datetime]
is_leap [year]
minute [datetime]
month [datetime]
now [format_string]
parse <datetime_string> [format_string]
second [datetime]
seconds [datetime]
sleep <duration>
stamp [datetime]
stamp_ms [datetime]
timezone [datetime] <offset_hours> [format_string]
to_string <datetime> [format_string]
weekday [datetime]
year [datetime]
### ui
confirm <msg>
date_pick [msg|cfg_map]
editor [msg|cfg_map]
float <msg> [decimal_places=2]
int <msg>
join_flow <max_width> <widgets...>
joinx <widget1> <widget2>
joiny <widget1> <widget2>
multi_pick <options> [msg|cfg_map]
password <msg> [confirm=false]
pick <options> [msg|cfg_map]
text <msg> [init_value]
widget <content> <title> [width] [height]
