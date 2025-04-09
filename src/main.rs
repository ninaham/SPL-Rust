//use parser::parser::alphanumeric;

use parser::parse_everything_else::parse;

pub mod absyn;
pub mod parser;


fn main() {
    let test = "
type LineBuffer = array[4096] of int;

type Term = array[2] of int;
type Memory = array[65535] of Term;
type DeBrujinEntry = array[32] of int;
type DeBrujinMap = array[2048] of DeBrujinEntry;

proc printLine(ref buffer: LineBuffer) {
	var i: int;
	var continue: int;

	continue := 1;
	i := 0;

	while (continue = 1) {
		if (buffer[i] # 0) {
			printc(buffer[i]);
			i := i + 1;
			if (i >= 4096) {
				continue := 0;
			}
		} else {
			continue := 0;
		}
	}

	printc('\n');
}

proc readLine(ref buffer: LineBuffer) {
	var c: int;
	var i: int;
	var continue: int;

	i := 0;
	continue := 1;

	while (continue = 1) {
		readc(c);

		if (c = 8) {
			if (i > 0) {
				i := i - 1;
				printc(c);
			}
		} else if (c = 127) {
			if (i > 0) {
				i := i - 1;
				printc(c);
			}
		} else if (c = '\n') {
			buffer[i] := 0;
			continue := 0;
		} else if (c = 13) {
			buffer[i] := 0;
			continue := 0;
		} else if (i = 4095) {
			printc(c);
			buffer[i] := 0;
			continue := 0;
		} else {
			printc(c);
			buffer[i] := c;
			i := i + 1;
		}
	}
}

proc parse(ref buffer: LineBuffer, ref memory: Memory, ref memorySize: int, ref insertion: int, ref success: int) {
	var i: int;
	var map: DeBrujinMap;
	var size: int;

	memorySize := 0;
	size := 0;
	i := 0;
	success := 1;
	parseAbs(buffer, memory, memorySize, insertion, map, size, i, success, 0);

	if (success = 1) {
		skipWhiteSpace(buffer, i);
		if (buffer[i] # 0) {
			printErrorUnexpectedInput(success, buffer[i]);
		}
	}
}

proc skipWhiteSpace(ref buffer: LineBuffer, ref i: int) {
	var continue: int;

	continue := 1;

	while (continue = 1) {
		if (buffer[i] = 0) {
			continue := 0;
		} else if (buffer[i] < 33) {
			i := i + 1;
		} else if (buffer[i] = 127) {
			i := i + 1;
		} else {
			continue := 0;
		}
	}
}

proc printErrorUnexpectedEndOfInput(ref success: int) {
	printc('['); printc('e'); printc('r'); printc('r'); printc('o'); printc('r'); printc(']'); printc(' ');
	printc('u'); printc('n'); printc('e'); printc('x'); printc('p'); printc('e'); printc('c'); printc('t');
	printc('e'); printc('d'); printc(' '); printc('e'); printc('n'); printc('d'); printc(' '); printc('o');
	printc('f'); printc(' '); printc('i'); printc('n'); printc('p'); printc('u'); printc('t'); printc('\n');

	success := 0;
}

proc printErrorOutOfMemory(ref success: int) {
	printc('['); printc('e'); printc('r'); printc('r'); printc('o'); printc('r'); printc(']'); printc(' ');
	printc('o'); printc('u'); printc('t'); printc(' '); printc('o'); printc('f'); printc(' '); printc('m');
	printc('e'); printc('m'); printc('o'); printc('r'); printc('y'); printc('\n');

	success := 0;
}

proc printErrorUnexpectedInput(ref success: int, input: int) {
	printc('['); printc('e'); printc('r'); printc('r'); printc('o'); printc('r'); printc(']'); printc(' ');
	printc('u'); printc('n'); printc('e'); printc('x'); printc('p'); printc('e'); printc('c'); printc('t');
	printc('e'); printc('d'); printc(' '); printc('i'); printc('n'); printc('p'); printc('u'); printc('t');
	printc(' ');
	printc(39);
	printc(input);
	printc(39);
	printc('\n');

	success := 0;
}

proc printErrorVariableNameTooLong(ref buffer: LineBuffer, start: int, length: int, ref success: int) {
	var i: int;

    printc(91); printc(101); printc(114); printc(114); printc(111); printc(114); printc(93); printc(32);
    printc(118); printc(97); printc(114); printc(105); printc(97); printc(98); printc(108); printc(101);
    printc(32); printc(110); printc(97); printc(109); printc(101); printc(32); printc(39);

    i := 0;
	while (i < length) {
		printc(buffer[start + i]);
		i := i + 1;
	}

    printc(39);
    printc(32); printc(109); printc(97); printc(121); printc(32); printc(110); printc(111); printc(116);
    printc(32); printc(101); printc(120); printc(99); printc(101); printc(101); printc(100); printc(32);
    printc(51); printc(48); printc(32); printc(99); printc(104); printc(97); printc(114); printc(97);
    printc(99); printc(116); printc(101); printc(114); printc(115); printc(32); printc(108); printc(101);
    printc(110); printc(103); printc(116); printc(104);
    printc('\n');

    success := 0;
}


proc printErrorUnknownVariable(ref buffer: LineBuffer, start: int, length: int, ref success: int) {
	var i: int;

	printc(91); printc(101); printc(114); printc(114); printc(111); printc(114); printc(93); printc(32);
    printc(117); printc(110); printc(107); printc(110); printc(111); printc(119); printc(110); printc(32);
    printc(118); printc(97); printc(114); printc(105); printc(97); printc(98); printc(108); printc(101);
    printc(32); printc(39);

    i := 0;
    while (i < length) {
    	printc(buffer[start + i]);
    	i := i + 1;
    }

    printc(39);
    printc('\n');

	success := 0;
}

proc skipVar(ref buffer: LineBuffer, ref i: int, ref success: int) {
	var continue: int;
	var first: int;

	first := 1;
	continue := 1;
	while (continue = 1) {
		if (buffer[i] = 0) {
			continue := 0;
		} else if (buffer[i] >= 'a') {
			if (buffer[i] <= 'z') {
				i := i + 1;
				first := 0;
			} else {
				continue := 0;
			}
		} else if (buffer[i] = '_') {
			i := i + 1;
			first := 0;
		} else if (buffer[i] >= 'A') {
			if (buffer[i] <= 'Z') {
				i := i + 1;
				first := 0;
			} else {
				continue := 0;
			}
		} else if (buffer[i] >= '0') {
			if (buffer[i] <= '9') {
				if (first # 1) {
					i := i + 1;
					first := 0;
				} else {
					continue := 0;
				}
			} else {
				continue := 0;
			}
		} else {
			continue := 0;
		}
	}

	if (first = 1) {
		if (buffer[i] = 0) {
			printErrorUnexpectedEndOfInput(success);
		} else {
			printErrorUnexpectedInput(success, buffer[i]);
		}
	}
}

proc getDeBrujinIndex(ref map: DeBrujinMap, ref size: int, binder: int, ref buffer: LineBuffer, start: int, length: int, ref success: int, ref result: int) {
	var i: int;
	var j: int;
	var found: int;

	i := size - 1;
	found := 0;

	while (i >= 0) {
		if (map[i][1] = length) {
			found := 1;
			j := 0;

			while (j < length) {
				if (map[i][j + 2] # buffer[start + j]) {
					found := 0;
					j := length - 1;
				}

				j := j + 1;
			}

			if (found = 1) {
				result := binder - map[i][0];
				i := 0;
			}
		}

		i := i - 1;
	}

	if (found = 0) {
		printErrorUnknownVariable(buffer, start, length, success);
	}
}

proc enterDeBrujinIndex(ref map: DeBrujinMap, ref size: int, binder: int, ref buffer: LineBuffer, start: int, length: int, ref success: int) {
	var i: int;

	if (size >= 2048) {
		printErrorOutOfMemory(success);
	} else {
		map[size][0] := binder;
		map[size][1] := length;

		i := 0;
		while (i < length) {
			map[size][i + 2] := buffer[start + i];
			i := i + 1;
		}

		size := size + 1;
	}
}

proc enterTerm(ref memory: Memory, ref memorySize: int, ref insertion: int, left: int, right: int, ref success: int) {
	if (memorySize >= 65535) {
		printErrorOutOfMemory(success);
	} else {
		insertion := memorySize;
		memorySize := memorySize + 1;

		memory[insertion][0] := left;
		memory[insertion][1] := right;
	}
}

proc parseAbs(ref buffer: LineBuffer, ref memory: Memory, ref memorySize: int, ref insertion: int, ref map: DeBrujinMap, ref size: int, ref i: int, ref success: int, binder: int) {
	var start: int;
	var length: int;
	var rhs: int;

	skipWhiteSpace(buffer, i);

	if (buffer[i] = 0) {
		printErrorUnexpectedEndOfInput(success);
	} else if (buffer[i] = 92) {
		i := i + 1;

		skipWhiteSpace(buffer, i);

		start := i;
		skipVar(buffer, i, success);

		if (success = 1) {
			length := i - start;

			if (length > 30) {
				printErrorVariableNameTooLong(buffer, start, length, success);
			} else {
				skipWhiteSpace(buffer, i);

				if (buffer[i] = 0) {
					printErrorUnexpectedEndOfInput(success);
				} else if (buffer[i] # '.') {
					printErrorUnexpectedInput(success, buffer[i]);
				} else {
					i := i + 1;
					enterDeBrujinIndex(map, size, binder, buffer, start, length, success);
					if (success = 1) {
						parseAbs(buffer, memory, memorySize, rhs, map, size, i, success, binder + 1);
						if (success = 1) {
							enterTerm(memory, memorySize, insertion, 0, rhs, success);
							if (success = 1) {
								size := size - 1;
							}
						}
					}
				}
			}
		}
	} else {
		parseApp(buffer, memory, memorySize, insertion, map, size, i, success, binder);
	}
}

proc parseApp(ref buffer: LineBuffer, ref memory: Memory, ref memorySize: int, ref insertion: int, ref map: DeBrujinMap, ref size: int, ref i: int, ref success: int, binder: int) {
	var lhs: int;
	var rhs: int;

	skipWhiteSpace(buffer, i);

	if (buffer[i] = 0) {
		printErrorUnexpectedEndOfInput(success);
	} else if (buffer[i] = '(') {
		i := i + 1;

		skipWhiteSpace(buffer, i);
		parseAbs(buffer, memory, memorySize, lhs, map, size, i, success, binder);

		if (success = 1) {
			skipWhiteSpace(buffer, i);
			parseAbs(buffer, memory, memorySize, rhs, map, size, i, success, binder);

			if (success = 1) {
				skipWhiteSpace(buffer, i);

				if (buffer[i] = 0) {
					printErrorUnexpectedEndOfInput(success);
				} else if (buffer[i] # ')') {
					printErrorUnexpectedInput(success, buffer[i]);
				} else {
					i := i + 1;
					enterTerm(memory, memorySize, insertion, -(lhs + 1), rhs, success);
				}
			}
		}
	} else if (buffer[i] >= '0') {
		if (buffer[i] <= '9') {
			parseNum(buffer, memory, memorySize, insertion, map, size, i, success, binder);
		} else {
			parseVar(buffer, memory, memorySize, insertion, map, size, i, success, binder);
		}
	} else {
		parseVar(buffer, memory, memorySize, insertion, map, size, i, success, binder);
	}
}

proc parseNum(ref buffer: LineBuffer, ref memory: Memory, ref memorySize: int, ref insertion: int, ref map: DeBrujinMap, ref size: int, ref i: int, ref success: int, binder: int) {
	var start: int;
	var j: int;
	var continue: int;
	var value: int;
	var x: int;
	var f: int;

	skipWhiteSpace(buffer, i);

	if (buffer[i] = 0) {
		printErrorUnexpectedEndOfInput(success);
	} else if (buffer[i] >= '0') {
		if (buffer[i] <= '9') {
			start := i;
			continue := 1;

			value := 0;
			while (continue = 1) {
				if (buffer[i] = 0) {
					continue := 0;
				} else if (buffer[i] < '0') {
					continue := 0;
				} else if (buffer[i] > '9') {
					continue := 0;
				} else {
					value := value * 10;
					value := value + (buffer[i] - '0');
					i := i + 1;
				}
			}

			enterTerm(memory, memorySize, x, 1, 1, success);
			if (success = 1) {
				enterTerm(memory, memorySize, f, 1, 2, success);
				if (success = 1) {
					j := 0;
					insertion := x;

					while (j < value) {
						enterTerm(memory, memorySize, insertion, -(f + 1), insertion, success);
						if (success = 0) {
							j := value - 1;
						}
						j := j + 1;
					}

					if (success = 1) {
						enterTerm(memory, memorySize, insertion, 0, insertion, success);
						if (success = 1) {
							enterTerm(memory, memorySize, insertion, 0, insertion, success);
						}
					}
				}
			}
		} else {
			printErrorUnexpectedInput(success, buffer[i]);
		}
	} else {
		printErrorUnexpectedInput(success, buffer[i]);
	}
}

proc parseVar(ref buffer: LineBuffer, ref memory: Memory, ref memorySize: int, ref insertion: int, ref map: DeBrujinMap, ref size: int, ref i: int, ref success: int, binder: int) {
	var start: int;
	var length: int;
	var index: int;

	skipWhiteSpace(buffer, i);

	if (buffer[i] = 0) {
		printErrorUnexpectedEndOfInput(success);
	} else {
		start := i;
		skipVar(buffer, i, success);

		if (success = 1) {
			length := i - start;
			getDeBrujinIndex(map, size, binder, buffer, start, length, success, index);

			if (success = 1) {
				enterTerm(memory, memorySize, insertion, 1, index, success);
			}
		}
	}
}

proc copyDeBrujin(ref memory: Memory, ref memorySize: int, ref index: int, ref success: int) {
	var fn: int;
	var insertion: int;

	if (memory[index][0] < 0) {
		if (memorySize >= 65535) {
			printErrorOutOfMemory(success);
		} else {
			insertion := memorySize;
			memorySize := memorySize + 1;

			memory[insertion][0] := memory[index][0];
			memory[insertion][1] := memory[index][1];

			fn := -memory[insertion][0] - 1;
			copyDeBrujin(memory, memorySize, fn, success);

			if (success = 1) {
				memory[insertion][0] := -(fn + 1);
				copyDeBrujin(memory, memorySize, memory[insertion][1], success);

				if (success = 1) {
					index := insertion;
				}
			}
		}
	} else if (memory[index][0] = 0) {
		if (memorySize >= 65535) {
			printErrorOutOfMemory(success);
		} else {
			insertion := memorySize;
			memorySize := memorySize + 1;

			memory[insertion][0] := memory[index][0];
			memory[insertion][1] := memory[index][1];
			copyDeBrujin(memory, memorySize, memory[insertion][1], success);

			if (success = 1) {
				index := insertion;
			}
		}
	} else {
		if (memorySize >= 65535) {
			printErrorOutOfMemory(success);
		} else {
			memory[memorySize][0] := memory[index][0];
			memory[memorySize][1] := memory[index][1];
			index := memorySize;
			memorySize := memorySize + 1;
		}
	}
}

proc substituteDeBrujin(ref memory: Memory, ref term: int, argument: int, binder: int) {
	var fn: int;

	if (memory[term][0] < 0) {
		fn := -memory[term][0] - 1;
		substituteDeBrujin(memory, fn, argument, binder);
		memory[term][0] := -(fn + 1);

		substituteDeBrujin(memory, memory[term][1], argument, binder);
	} else if (memory[term][0] = 0) {
		substituteDeBrujin(memory, memory[term][1], argument, binder + 1);
	} else if (memory[term][1] > binder) {
		memory[term][1] := memory[term][1] - 1;
	} else if (memory[term][1] = binder) {
		term := argument;
	}
}

proc applyDeBrujin(ref memory: Memory, ref memorySize: int, ref function: int, argument: int, ref success: int) {
	copyDeBrujin(memory, memorySize, function, success);
	if (success = 1) {
		substituteDeBrujin(memory, function, argument, 0);
		function := memory[function][1];
	}
}";
    let n = parse(test);
    println!("{:#?}", n);
}
