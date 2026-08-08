### - Top level Builtin Functions
- assert <expr> [expr] [message]
	throw if not equal/truthy
- cd [path=~]
	change dir. '-' for previous
- cwd 
	current dir
- ddebug <args>...
	eval & show expr,type,value(pretty fmt)
- debug <args>...
	eval & show expr,type,value(debug fmt)
- dig <map|list|set|range|table> <path>
	get nested value by dot path. e.g. dig m 'a.b.0'
- eprint <args>...
	print to stderr, red, no newline
- eprintln <args>...
	print to stderr, red, with newline
- eval <expr>
	eval expr in current env
- eval_str <string>
	parse & eval string in current env
- exec <expr>
	eval expr in forked env
- exec_str <string>
	parse & eval string in forked env
- exit [status=0]
	exit shell
- flatten <collection>
	flatten nested list/map to flat list
- format <template> <args>...
	fmt string. {name}/{} for named/positional, :spec for align.
	e.g. format '{:0>5}' 3 -> 00003
- get_env <var>
	get var from env
- get_local <var>
	get local var value
- get_var <var>
	get var, local first then env
- help [libs|tops|doc|<lib>|<lib>.<func>|<top_func>]
	show help. e.g. help string.split
- import <path>
	eval file in forked env
- include <path>
	eval file in current env
- jobs [-k id]
	list/kill jobs
- len <list|set|map|table|range|string|bytes>
	size of collection
- not <boolean>...
	logic not
- pprint <value>...
	pretty print table/list/map
- print <args>...
	print, space-sep, no newline
- println <args>...
	print, space-sep, with newline
- quote <expr>
	quote expr, eval later
- read [-p prompt] [-n max_chars] [-s silent] [-t timeout_secs]
	read input
- repeat <expr> <n>
	eval expr n times, collect non-None results
- rev <string|list|table|bytes>
	reverse
- select <table> <columns...>
	select columns from table
- set_root <var> <val>
	define var in root env
- symof <value>
	type name before eval
- tap <args>...
	print then return value(s)
- throw <msg>
	raise a runtime error
- typeof <value>
	type name after eval
- unset_root <var>
	undefine var in root env
- when <condition> <execute>
	conditional execute
- where <table> <condition>
	filter table rows. NR/<col_name> injected. e.g. where t (NR>1 and col>0)

### about
- bin 
	bin path
- history 
	history path
- info 
	all info
- prelude 
	prelude path
- version 
	version

### boolean
- and <boolean1>...
	logic and
- not <boolean1>...
	logic not
- or <boolean1>...
	logic or

### bytes
- concat <bytes> <bytes>
	concat two bytes
- contains <bytes> <bytes>
	contains sub-sequence?
- ends_with <bytes> <bytes>
	ends with suffix?
- from <string>
	bytes from utf8 string
- from_base64 <base64_string>
	bytes from base64 string
- from_escaped <string>
	bytes from escaped text. e.g. '\n\x41'
- from_hex <hex_string>
	bytes from hex string, '0x' prefix ok
- from_list <list>
	bytes from int(0-255) list
- is_empty <bytes>
	is empty?
- len <bytes>
	byte length
- pop <bytes>
	drop last byte, returns new bytes
- position <bytes> <bytes>
	index of sub-sequence, -1 if absent
- push <bytes> <int>
	append byte(0-255), returns new bytes
- repeat <bytes> <n>
	repeat n times
- reverse <bytes>
	reverse bytes
- slice <bytes> <start> <end>
	sub-bytes [start,end)
- split <bytes> <int>
	split by byte value(0-255)
- starts_with <bytes> <bytes>
	starts with prefix?
- to_base64 <bytes>
	to base64 string
- to_hex <bytes>
	to hex string
- to_list <bytes>
	to list of int(0-255)
- to_string <bytes>
	to utf8 string (lossy)

### console
- alt_screen <bool>
	enter/leave alternate screen
- bell 
	ring terminal bell
- clear 
	clear console
- cursor_down <n>
	move cursor down n rows
- cursor_hide 
	hide cursor
- cursor_left <n>
	move cursor left n cols
- cursor_restore 
	restore cursor position
- cursor_right <n>
	move cursor right n cols
- cursor_save 
	save cursor position
- cursor_show 
	show cursor
- cursor_to <x> <y>
	move cursor to position
- cursor_up <n>
	move cursor up n rows
- discard <args>...
	no-op, discards args
- flush 
	flush stdout
- height 
	console height
- keys 
	list special key names
- line_wrap <bool>
	enable/disable line wrap
- print_tty <text>
	write raw text directly to tty, bypass pipes
- raw_mode [bool]
	get/set raw mode
- read_key 
	read one key, enters raw mode temporarily. e.g. 'enter','f1','a'
- read_line [prompt]
	read line from stdin
- read_password [prompt]
	read password, masked
- title <string>
	set console title
- width 
	console width
- write <text> <x> <y>
	write text at position

### filesize
- b <filesize>
	bytes
- from <size_str|byte_int>
	to Filesize
- gb <filesize>
	gigabytes
- kb <filesize>
	kilobytes (integer, truncated)
- mb <filesize>
	megabytes
- tb <filesize>
	terabytes
- to_string <filesize>
	to human readable string

### from
- cmd <output> [split_regex] [headers...]
	parse cmd output into table
- csv <csv_string>
	parse CSV string, headers row required
- jq <json_string> <query_string>
	jq-like query on json string. e.g. '.a|.[]|select(.n>1)'
- json <json_string>
	parse JSON string
- script <script_string>
	parse script text to expression (unevaluated)
- toml <toml_string>
	parse TOML string

### fs
- abs <path>
	absolute path
- append <content> <file>
	append to file, creates if missing
- base_name <path>
	file name with extension
- canon <path>
	canonical path, resolves symlinks
- chmod <path> <mode:octal>
	set unix permission mode
- chown <path> <uid> <gid>
	set unix owner, -1 to keep
- cp <source> <destination>
	copy path, recursive for dirs
- dir_name <path>
	dir part before last '/'
- exists <path>
	path exists?
- extension <path>
	file extension
- glob <pattern>
	match files by pattern
- head <file> [n=10]
	first n lines
- is_dir <path>
	is dir?
- is_file <path>
	is file?
- join <segment>...
	join path segments
- ls [-l|a|h|t|L|c|u|m|p|?] [path]
	list dir contents
- mkdir <path>
	create dir, incl parents
- mv <source> <destination>
	move path
- parent <path>
	parent dir path
- read <file>
	read file, text or bytes
- read_link <link_path>
	read symlink target
- rm <path>
	remove path, recursive for dirs
- rmdir <path>
	remove empty dir
- stem <path>
	file name without extension
- symlink <source> <link_path>
	create symlink
- tail <file> [n=10]
	last n lines
- touch <path>
	create empty file, or update mtime if exists
- tree [depth=3] [path]
	dir tree as nested map
- write [content] <file>
	create/overwrite file

### hmap
- contains_key <map> <key>
	has key?
- contains_value <map> <value>
	has value?
- difference <map1> <map2>
	keys in map1 not in map2
- dig <map|list|range> <path>
	get nested value by dot path. e.g. dig m 'a.b.0'
- filter <map> <fn>
	keep pairs where fn(k,v)->bool
- find <map> <fn>
	first pair matching fn(k,v)->bool, returns [k,v]
- flatten <map>
	flatten nested structure
- from_list <list>
	create map from list of [k,v] pairs
- get <map> <key>
	value by key
- insert <map> <key> <value>
	insert key-value, returns new map
- intersection <map1> <map2>
	keys in both, values from map1
- is_empty <map>
	is empty?
- keys <map>
	list of keys
- len <map>
	map size
- map <map> <entry_fn>
	transform keys/values, fn(k,v)->[k,v]
- merge <map1> <map2> [<map3>...]
	deep merge maps, recurse on nested maps
- remove <map> <key>
	remove key, returns new map
- set <map> <key> <value>
	set existing key's value, returns new map
- to_bmap <map>
	to BtreeMap (ordered)
- to_list <map>
	to list of [k,v] pairs
- union <map1> <map2>
	combine maps, map2 wins on conflict
- values <map>
	list of values

### into
- boolean <value>
	to bool
- caesar <string> [shift=13]
	caesar cipher
- csv <expr>
	to CSV
- filesize <size_str|int>
	to filesize. e.g. 1.5GB, 500K
- float <str|num|bool>
	to float. % as /100, _ as sep. e.g. 12.5% -> 0.125
- highlight <script>
	ANSI highlight script
- int <str|num|bool>
	to int. radix ok(0x/0o/0b), _ as sep. e.g. 0xff_80
- json <expr>
	to JSON
- pretty <expr>
	to pretty string
- safe <str>
	wrap str, never eval
- string <value>
	to string
- strip <string>
	remove ANSI codes
- table <output> [split_regex] [headers...]
	parse cmd output to table
- time <str> [fmt]
	to datetime
- toml <expr>
	to TOML

### list
- all <list> <fn>
	all elements pass?
- any <list> <fn>
	any element passes?
- average <num1> <num2>... | <array>
	average of numbers
- chunks <list> <size>
	split into chunks of size n
- concat <list1|item1> <list2|item2>...
	concat lists/items into one list
- contains <list> <item>
	contains item?
- dig <map|list|range> <path>
	get nested value by dot path. e.g. dig m 'a.b.0'
- fill <value> <n>
	repeat value n times
- filter <list> <fn>
	filter by fn([index],item)
- filter_map <list> <fn>
	filter+map, drop None results
- find <list> <item|fn> [skip_n=0]
	first matched item
- first <list> [n=1]
	first n elements
- flatten <collection>
	flatten nested structure
- fold <list> <fn> [init=0]
	fold left, fn(acc,item)
- from <range>
	list from range
- get <list> <index>
	nth element, negative index from end
- group <list> <key_fn|key>
	group by key fn or map field, e.g.  fn(item)->string
- insert <list> <index> <value>
	insert value at index
- is_empty <list>
	is empty?
- items <list>
	index-value pairs
- join <list> <separator>
	join strings with separator
- last <list> [n=1]
	last n elements
- len <list>
	list length
- map <list> <fn>
	apply fn([index],item) per element
- max <num1> <num2>... | <array>
	max value
- min <num1> <num2>... | <array>
	min value
- position <list> <item|fn> [skip_n=0]
	first matched index
- push <list> <element>
	append element
- remove <list> <item> [all=false]
	remove item, default first-only
- remove_at <list> <index> [count=1]
	remove n items from index
- rev <list>
	reverse
- rfind <list> <item|fn> [skip_n=0]
	last matched item
- rfold <list> <fn> [init=0]
	fold right, fn(acc,item)
- rotate <list> <n>
	rotate, n>0 right, n<0 left
- rposition <list> <item|fn> [skip_n=0]
	last matched index
- sample <list> <n>
	pick n distinct random elements
- set <list> <index> <value>
	set value at existing index
- shuffle <list>
	shuffle order
- skip <list> <count>
	skip first n elements
- slice <list> <start> <end>
	sub-list [start,end), negative index ok
- sort <list> [key_fn|±key...]
	sort, optional fn(a,b)->[-1/0/1]. e.g. sort list 'name'
- splice <list> <start> <delete_count> [items...]
	delete & optionally insert at index, returns new list
- split_at <list> <index>
	split at index, returns [left,right]
- split_first <list>
	split head/tail, returns [head,rest]
- sum <num1> <num2>... | <array>
	sum of numbers
- swap <list> <i> <j>
	swap two elements by index
- take <list> <count>
	first n elements
- to_hmap <list> [key_fn] [val_fn]
	to hashMap, default pairs [k,v,k,v...]
- to_map <list> [key_fn] [val_fn]
	to btreeMap, default pairs [k,v,k,v...]
- to_set <list>
	to btreeSet
- transpose <matrix>
	transpose matrix (list of lists)
- unique <list>
	dedupe, preserve order
- unzip <list_of_pairs>
	unzip pairs into two lists
- windows <list> <size>
	overlapping sliding windows of size n
- zip <list1> <list2>
	zip two lists into pairs

### log
- debug <msg>
	log debug
- disable 
	disable all log
- echo <msg>
	print message without formatting
- enable [int]
	enable all log, or level
- error <msg>
	log error
- info <msg>
	log info
- is_enabled <level>
	log level is enabled?
- level [int]
	get/set the log level
- trace <msg>
	log trace
- warn <msg>
	log warning

### map
- contains_key <map> <key>
	has key?
- contains_value <map> <value>
	has value?
- difference <map1> <map2>
	keys in map1 not in map2
- dig <map|list|range> <path>
	get nested value by dot path. e.g. dig m 'a.b.0'
- filter <map> <fn>
	keep pairs where fn(k,v)->bool
- find <map> <fn>
	first pair matching fn(k,v)->bool, returns [k,v]
- first <map>
	first key-value pair by key order, returns [k,v]
- flatten <map>
	flatten nested structure
- from_list <list>
	create map from list of [k,v] pairs
- get <map> <key>
	value by key
- insert <map> <key> <value>
	insert key-value, returns new map
- intersection <map1> <map2>
	keys in both, values from map1
- is_empty <map>
	is empty?
- keys <map>
	list of keys
- last <map>
	last key-value pair by key order, returns [k,v]
- len <map>
	map size
- map <map> <map_fn>
	transform keys/values, fn(k,v)->[k,v]
- merge <map1> <map2> [<map3>...]
	deep merge maps, recurse on nested maps
- remove <map> <key>
	remove key, returns new map
- set <map> <key> <value>
	set existing key's value, returns new map
- to_hmap <map>
	to hashMap (unordered)
- to_list <map>
	to list of [k,v] pairs
- union <map1> <map2>
	combine maps, map2 wins on conflict
- values <map>
	list of values

### math
- abs <number>
	absolute value
- acos <value>
	inverse cosine
- acosh <value>
	inverse hyperbolic cosine
- asin <value>
	inverse sine
- asinh <value>
	inverse hyperbolic sine
- atan <value>
	inverse tangent
- atanh <value>
	inverse hyperbolic tangent
- average <num1> <num2>... | <array>
	average of numbers
- bit_and <int1> <int2>
	bitwise AND
- bit_not <integer>
	bitwise NOT
- bit_or <int1> <int2>
	bitwise OR
- bit_shl <integer> <bits>
	shift left, bits 0-63
- bit_shr <integer> <bits>
	shift right, bits 0-63
- bit_xor <int1> <int2>
	bitwise XOR
- cbrt <number>
	cube root
- ceil <number>
	round up
- clamp <value> <min> <max>
	clamp value into [min,max]
- cos <radians>
	cosine
- cos_pi <x>
	cos(x*π)
- cosh <value>
	hyperbolic cosine
- eq <a> <b>
	a == b?
- exp <x>
	e^x
- exp2 <x>
	2^x
- floor <number>
	round down
- gcd <int1> <int2>
	greatest common divisor
- ge <a> <b>
	a >= b?
- gt <a> <b>
	a > b?
- hypot <x> <y>
	sqrt(x^2+y^2)
- is_even <integer>
	is even?
- is_odd <integer>
	is odd?
- lcm <int1> <int2>
	least common multiple
- le <a> <b>
	a <= b?
- ln <number>
	natural log
- log <number> <base>
	log base of number
- log10 <number>
	log base 10
- log2 <number>
	log base 2
- lt <a> <b>
	a < b?
- max <num1> <num2>... | <array>
	max value
- min <num1> <num2>... | <array>
	min value
- ne <a> <b>
	a != b?
- pow <base> <exponent>
	base^exponent
- rem <a> <b>
	euclidean remainder
- round <number>
	round to nearest
- signum <number>
	sign: -1, 0, or 1
- sin <radians>
	sine
- sin_pi <x>
	sin(x*π)
- sinh <value>
	hyperbolic sine
- sqrt <number>
	square root
- sum <num1> <num2>... | <array>
	sum of numbers
- tan <radians>
	tangent
- tan_pi <x>
	tan(x*π)
- tanh <value>
	hyperbolic tangent
- to_degrees <radians>
	radians to degrees
- to_radians <degrees>
	degrees to radians
- to_string <number>
	to string
- trunc <number>
	truncate decimal

### rand
- alpha [len=1]
	random alphabetic char(s)
- alphanum [len=1]
	random alphanumeric char(s)
- chance [p=0.5]
	random bool with probability p
- choose <list>
	pick random item
- float [min] [max]
	random float. no args: [0,1); 2 args: [min,max]
- int [min] [max]
	random integer. no args: any i64; 1 arg: [0,max]; 2 args: [min,max]
- ratio <num> <den>
	random bool with probability num/den
- sample <list> <n>
	pick n distinct items, no replacement
- seed <integer>
	seed generator for reproducible sequence
- shuffle <list>
	shuffle order, returns new list

### regex
- capture <pattern> <text>
	first match's groups [full,g1,g2,...]
- captures <pattern> <text>
	all matches' groups [[full,g1,...],...]
- find <pattern> <text>
	first match, returns {start,end,found}
- find_all <pattern> <text>
	all matches, list of {start,end,found}
- from <pattern_string> <flags>
	build regex from string and flags
- is_match <pattern> <text>
	contains a match?
- named_captures <pattern> <text>
	named groups as map. e.g. g'(?<y>\d+)'
- replace <text> <pattern> <replacement>
	replace first match
- replace_all <text> <pattern> <replacement>
	replace all matches
- split <pattern> <text>
	split by pattern

### set
- all <set> <fn>
	all items pass fn(item)->bool?
- any <set> <fn>
	any item passes fn(item)->bool?
- contains <set> <item>
	contains item?
- difference <set1> <set2>
	items in set1 not in set2
- filter <set> <fn>
	keep items where fn(item)->bool
- find <set> <fn>
	first item matching fn(item)->bool
- first <set>
	smallest item
- from_list <list>
	create set from list
- get <set> <index>
	nth element, negative index from end
- insert <set> <item>
	add item, returns new set
- intersection <set1> <set2>
	intersection
- is_disjoint <set1> <set2>
	no common items?
- is_empty <set>
	is empty?
- is_subset <set1> <set2>
	set1 ⊆ set2?
- is_superset <set1> <set2>
	set1 ⊇ set2?
- last <set>
	largest item
- len <set>
	set size
- map <set> <fn>
	apply fn(item)->new_item to each
- remove <set> <item>
	remove item, returns new set
- split_first <set>
	pop smallest, returns [item,rest]
- split_last <set>
	pop largest, returns [item,rest]
- symmetric_difference <set1> <set2>
	items in either but not both
- to_list <set>
	to list, sorted order
- union <set1> <set2>
	union

### string
- black <string>
	black fg
- blink <string>
	blink
- blue <string>
	blue fg
- bold <string>
	bold
- center <string> <length> [pad_char=' ']
	pad both ends
- chars <string>
	to char list
- clr <string> <0..256>
	256-color fg, code 0-255
- clr_bg <string> <0..256>
	256-color bg, code 0-255
- color <string> <#hex|name|r,g,b>
	true color fg. e.g. #ff0000, red, 255,0,0
- color_bg <string> <#hex|name|r,g,b>
	true color bg. e.g. #ff0000, red, 255,0,0
- colors [swatches?]
	list color names, or with swatches
- concat <string>...
	join strings
- contains <string> <substring>
	contains?
- cyan <string>
	cyan fg
- dim <string>
	dim
- ends_with <string> <substring>
	ends with?
- escape <string>
	escape control chars to \n \t \xNN etc.
- get <string> <index>
	char at index. negative counts from end
- green <string>
	green fg
- grep <string> <substring>
	lines matching substring
- href <url> <text>
	terminal hyperlink
- insert <string> <index> <string>
	insert string at index
- invert <string>
	invert fg/bg
- is_alpha <string>
	is alphabetic?
- is_alphanumeric <string>
	is alphanumeric?
- is_ascii <string>
	is ascii?
- is_ascii_control <string>
	is ascii control char?
- is_ascii_digit <string>
	is ascii digit?
- is_ascii_hexdigit <string>
	is ascii hexdigit?
- is_ascii_punctuation <string>
	is ascii punctuation?
- is_empty <string>
	is empty?
- is_lower <string>
	is lowercase?
- is_numeric <string>
	is numeric?
- is_title <string>
	is title case?
- is_upper <string>
	is uppercase?
- is_whitespace <string>
	is whitespace?
- italic <string>
	italic
- len <string>
	char count
- lines <string>
	to line list
- lower <string>
	to lowercase
- magenta <string>
	magenta fg
- max_len <string>
	max line length
- pad_end <string> <length> [pad_char=' ']
	pad at end
- pad_start <string> <length> [pad_char=' ']
	pad at start
- paragraphs <string>
	to paragraph list
- position <string> <substring> [start]
	index of substring, or None. search from [start]
- red <string>
	red fg
- repeat <string> <count>
	repeat n times
- replace <string> <old> <new>
	replace all matches
- rev <string>
	reverse
- slice <string> <start> [end]
	substring [start,end)
- sort <string> ['+'|'-'|key_fn]
	sort lines
- split <string> [delimiter]
	split by delimiter/whitespace
- split_at <string> <index>
	split at index
- starts_with <string> <substring>
	starts with?
- strike <string>
	strikethrough
- strip_ansi <string>
	remove ANSI codes
- strip_prefix <string> <prefix>
	remove prefix
- strip_suffix <string> <suffix>
	remove suffix
- title <string>
	to title case
- to_filesize <size_str>
	to filesize. e.g. 1.5GB, 500K
- to_float <value>
	to float. % as /100, _ as sep. e.g. 12.5%
- to_int <value>
	to int. radix ok(0x/0o/0b), _ as sep. e.g. 0xff_80
- to_safe <str>
	wrap str, never eval
- to_table <output> [regex] [headers...]
	parse cmd output to table
- to_time <str> [fmt]
	to datetime
- trim <string>
	trim both ends
- trim_end <string>
	trim end
- trim_start <string>
	trim start
- underline <string>
	underline
- unescape <string>
	reverse of escape, parses \n \t \xNN \uXXXX
- upper <string>
	to uppercase
- white <string>
	white fg
- words <string>
	to word list
- words_quoted <string>
	to word list, quoted as one
- wrap <string> <width>
	wrap to width
- yellow <string>
	yellow fg

### sys
- defined <var>
	defined in scope chain?
- dirs 
	system directories map
- error_codes 
	list Lume error codes
- env [var]
	root env map, or var value
- has <var>
	defined in current scope?
- info 
	os info
- max_runtime [depth]
	get/set max runtime recursion depth
- max_syntax [depth]
	get/set max syntax recursion depth
- max_usemode [depth]
	get/set max use-mode recursion depth
- modes 
	current mode flags {cfm,strict,pdm}
- set_cfm <boolean|none>
	set Cmd First Mode
- set_pdm <boolean>
	enable/disable print direct mode
- set_strict <boolean>
	enable/disable strict mode
- vars 
	vars defined in current scope

### table
- filter <table> <cell|fn>
	filter rows by cell/fn(row_map)->bool
- first <table> [n=1]
	first n rows as lists
- first_map <table> [n=1]
	first n rows as maps
- from_maps <list_of_maps>
	build table from maps, headers = union of keys
- get <table> <index>
	nth row as list
- get_cell <table> <row_index> <header|index>
	single cell, negative row index ok
- get_column <table> <header|index>
	column by header/index
- get_map <table> <index>
	nth row as map
- grep <table> <string>
	rows containing string
- header_len <table>
	column count
- headers <table>
	list headers
- is_empty <table>
	has no rows?
- last <table> [n=1]
	last n rows as lists
- last_map <table> [n=1]
	last n rows as maps
- len <table>
	row count
- position <table> <cell|fn> [start=0]
	first row index matching cell/fn(row_map)->bool
- push <table> <list|set>
	append a row
- rows <table>
	rows as lists
- rows_map <table>
	rows as maps
- rposition <table> <cell|fn> [start=0]
	last row index matching cell/fn(row_map)->bool
- select <table> <cols...>
	select columns
- slice <table> <start> <end>
	row range as maps [start,end), negative index ok
- sort <list> [key_fn|±key...]
	sort, optional fn(a,b)->[-1/0/1]. e.g. sort table 'name'
- to_csv <table>
	serialize to CSV

### time
- add [datetime] <duration>
	add a signed duration string (e.g. '1d2h30m', '-1h') or integer seconds to a datetime (defaults to now)
- day [datetime]
	get day of month (1-31)
- diff <datetime1> [datetime2] <unit>
	calculate difference between two datetimes in given unit
- display [datetime]
	get preformatted datetime as map with time/date/datetime/etc.
- fmt [datetime] <format_string>
	format datetime (current or specified) using chrono format string
- hour [datetime]
	get hour (0-23)
- is_leap [year]
	check if a year is a leap year
- minute [datetime]
	get minute (0-59)
- month [datetime]
	get month (1-12)
- now [format_string]
	get current datetime as DateTime object or formatted string
- parse <datetime_string> [format_string]
	parse datetime string, optionally with a chrono format string
- second [datetime]
	get second (0-59)
- seconds [datetime]
	get seconds since midnight
- sleep <duration>
	sleep for a given number of milliseconds [ms] or duration string (e.g. '1s', '2m')
- stamp [datetime]
	get Unix timestamp in seconds
- stamp_ms [datetime]
	get Unix timestamp in milliseconds
- timezone [datetime] <offset_hours> [format_string]
	convert datetime to a different timezone offset (in hours)
- to_string <datetime> [format_string]
	convert DateTime to string
- weekday [datetime]
	get weekday (1-7, Monday=1)
- year [datetime]
	get year (current or from specified datetime)

### ui
- confirm <msg>
	ask yes/no
- date_pick [msg|cfg_map]
	pick a date.
	cfg: {msg,starting_date,min_date,max_date,week_start,formatter,validator}
- editor [msg|cfg_map]
	open external editor for multiline text.
	cfg: {msg,predefined_text,editor_command,validators...}
- float <msg> [decimal_places=2]
	read a float
- int <msg>
	read an int
- join_flow <max_width> <widgets...>
	flow-wrap widgets into rows
- joinx <widget1> <widget2>
	join two widgets side by side
- joiny <widget1> <widget2>
	stack two widgets vertically
- multi_pick <options> [msg|cfg_map]
	select multi, same options/cfg as pick.
	cfg Adds: {all_selected_by_default,keep_filter,validator}
- password <msg> [confirm=false]
	read password, masked
- pick <options> [msg|cfg_map]
	select one from list/set/range/map/table/glob/string.
	cfg: {msg,page_size,vim_mode,formatter,scorer,sorter...}
- text <msg> [init_value]
	read text
- widget <content> <title> [width] [height]
	draw a bordered text box
