/// Flat u32 opcode encoding.
///
/// Word 0: low 8 bits = opcode, high 24 bits = first operand (op0)
/// Multi-word instructions have additional 32-bit words for extra operands.
///
/// Instruction widths:
///   1-word: most instructions
///   2-word: COMPARE_JUMP, CALL_NAMED, INVOKE
///   3-word: LOCAL_COMPARE_JUMP, GLOBAL_COMPARE_JUMP, INVOKE_NAMED, ITER_NEXT, LOCAL_JUMP_IF_NE_CONST, FOR_LOOP_STEP
pub mod op {
    pub const INC_LOCAL: u8 = 0;
    pub const LOCAL_COMPARE_JUMP: u8 = 1;   // 3-word: op0=slot, w1=const_idx, w2=offset
    pub const COMPARE_JUMP: u8 = 2;         // 2-word: op0=const_idx, w1=offset
    pub const INC_GLOBAL: u8 = 3;
    pub const GLOBAL_COMPARE_JUMP: u8 = 4;  // 3-word: op0=name_idx, w1=const_idx, w2=offset
    pub const GET_LOCAL: u8 = 5;
    pub const SET_LOCAL: u8 = 6;
    pub const SET_LOCAL_POP: u8 = 7;
    pub const CONSTANT: u8 = 8;
    pub const ADD_INT: u8 = 9;
    pub const ADD_FLOAT: u8 = 10;
    pub const ADD: u8 = 11;
    pub const SUBTRACT: u8 = 12;
    pub const SUB_INT: u8 = 13;
    pub const SUB_FLOAT: u8 = 14;
    pub const MULTIPLY: u8 = 15;
    pub const MUL_INT: u8 = 16;
    pub const MUL_FLOAT: u8 = 17;
    pub const DIVIDE: u8 = 18;
    pub const DIV_FLOAT: u8 = 19;
    pub const POP: u8 = 20;
    pub const JUMP_IF_FALSE: u8 = 21;
    pub const JUMP: u8 = 22;
    pub const LOOP: u8 = 23;
    pub const RETURN: u8 = 24;
    pub const GET_GLOBAL: u8 = 25;
    pub const SET_GLOBAL: u8 = 26;
    pub const SET_GLOBAL_POP: u8 = 27;
    pub const DEFINE_GLOBAL: u8 = 28;
    pub const GET_PRIVATE: u8 = 29;
    pub const SET_PRIVATE: u8 = 30;
    pub const DUP: u8 = 31;
    pub const SWAP: u8 = 32;
    pub const OVER: u8 = 33;
    pub const INC: u8 = 34;
    pub const DEC: u8 = 35;
    pub const ARRAY: u8 = 36;
    pub const STRUCT: u8 = 37;
    pub const INDEX: u8 = 38;
    pub const SET_INDEX: u8 = 39;
    pub const MEMBER: u8 = 40;
    pub const SET_MEMBER: u8 = 41;
    pub const INC_MEMBER: u8 = 42;
    pub const STRING_CONCAT: u8 = 43;
    pub const CALL: u8 = 44;
    pub const CALL_NAMED: u8 = 45;              // 2-word: op0=total_count, w1=names_idx
    pub const INVOKE: u8 = 46;                  // 2-word: op0=name_idx, w1=arg_count
    pub const INVOKE_NAMED: u8 = 47;            // 3-word: op0=name_idx, w1=total_count, w2=names_idx
    pub const NEW: u8 = 48;
    pub const EQUAL: u8 = 49;
    pub const NOT_EQUAL: u8 = 50;
    pub const LESS: u8 = 51;
    pub const LESS_EQUAL: u8 = 52;
    pub const GREATER: u8 = 53;
    pub const GREATER_EQUAL: u8 = 54;
    pub const NOT: u8 = 55;
    pub const ITER_NEXT: u8 = 56;               // 3-word: op0=collection_slot, w1=cursor_slot|(has_index<<31), w2=exit_offset
    pub const LOCAL_JUMP_IF_NE_CONST: u8 = 57;  // 3-word: op0=slot, w1=const_idx, w2=offset
    pub const PUSH_HANDLER: u8 = 58;
    pub const POP_HANDLER: u8 = 59;
    pub const THROW: u8 = 60;
    pub const PRINT: u8 = 61;
    pub const PRINTLN: u8 = 62;
    pub const FOR_LOOP_STEP: u8 = 63;  // 3-word: op0=slot, w1=const_idx, w2=offset — increment local, compare < const, jump back if true
}
