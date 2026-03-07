use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpCode {
    // Hot Loop / Specialized Opcodes
    OpIncLocal(usize),
    OpLocalCompareJump(usize, usize, usize),
    OpCompareJump(usize, usize),
    OpIncGlobal(usize),
    OpGlobalCompareJump(usize, usize, usize),

    // Basic Hot Opcodes
    OpGetLocal(usize),
    OpSetLocal(usize),
    OpSetLocalPop(usize),
    OpConstant(usize),
    OpAddInt,
    OpAddFloat,
    OpAdd,
    OpSubtract,
    OpSubInt,
    OpSubFloat,
    OpMultiply,
    OpMulInt,
    OpMulFloat,
    OpDivide,
    OpDivFloat,
    OpPop,
    OpJumpIfFalse(usize),
    OpJump(usize),
    OpLoop(usize),
    OpReturn,

    // Global / Scope Opcodes
    OpGetGlobal(usize),
    OpSetGlobal(usize),
    OpSetGlobalPop(usize),
    OpDefineGlobal(usize),
    OpGetPrivate(usize),
    OpSetPrivate(usize),

    // Stack Manipulation
    OpDup,
    OpSwap,
    OpOver,
    OpInc,
    OpDec,

    // Data Structures
    OpArray(usize),
    OpStruct(usize),
    OpIndex,
    OpSetIndex,
    OpMember(usize),
    OpSetMember(usize),
    OpIncMember(usize),
    OpStringConcat,

    // Calls / Invocations
    OpCall(usize),
    OpCallNamed(usize, usize),
    OpInvoke(usize, usize),
    OpInvokeNamed(usize, usize, usize),
    OpNew(usize),

    // Comparison
    OpEqual,
    OpNotEqual,
    OpLess,
    OpLessEqual,
    OpGreater,
    OpGreaterEqual,

    // Control Flow / Misc
    OpIterNext(usize, usize, usize, bool),
    OpLocalJumpIfNeConst(usize, usize, usize),
    OpPushHandler(usize),
    OpPopHandler,
    OpThrow,
    OpPrint(usize),
    OpPrintln(usize),
}
