//! Disassembly tests.

use pulley_interpreter::*;

fn encoded(ops: &[Op]) -> Vec<u8> {
    let mut encoded = vec![];
    for op in ops {
        op.encode(&mut encoded);
    }
    log::trace!("encoded: {encoded:?}");
    encoded
}

fn assert_disas(ops: &[Op], expected: &str) {
    let expected = expected.trim();
    eprintln!("=== expected ===\n{expected}");

    let bytecode = encoded(ops);

    let actual = disas::Disassembler::disassemble_all(&bytecode).expect("decoding failed");
    let actual = actual.trim();
    eprintln!("=== actual ===\n{actual}");

    assert_eq!(expected, actual);
}

#[test]
fn simple() {
    assert_disas(
        &[
            // Prologue.
            Op::Xconst8(Xconst8 {
                dst: XReg::x27,
                imm: -16i8,
            }),
            Op::Xadd32(Xadd32 {
                operands: BinaryOperands {
                    dst: XReg::x1,
                    src1: XReg::x2,
                    src2: XReg::x27,
                },
            }),
            Op::Store64Offset8(Store64Offset8 {
                ptr: XReg::x1,
                offset: 8,
                src: XReg::x2,
            }),
            Op::Store64(Store64 {
                ptr: XReg::x1,
                src: XReg::x2,
            }),
            Op::Xmov(Xmov {
                dst: XReg::x1,
                src: XReg::x2,
            }),
            // Function body.
            Op::Xadd32(Xadd32 {
                operands: BinaryOperands {
                    dst: XReg::x0,
                    src1: XReg::x0,
                    src2: XReg::x1,
                },
            }),
            // Epilogue.
            Op::Xmov(Xmov {
                dst: XReg::x1,
                src: XReg::x2,
            }),
            Op::Load64Offset8(Load64Offset8 {
                dst: XReg::x1,
                ptr: XReg::x2,
                offset: 8,
            }),
            Op::Load64(Load64 {
                dst: XReg::x1,
                ptr: XReg::x2,
            }),
            Op::Xconst8(Xconst8 {
                dst: XReg::x27,
                imm: 16,
            }),
            Op::Xadd32(Xadd32 {
                operands: BinaryOperands {
                    dst: XReg::x1,
                    src1: XReg::x2,
                    src2: XReg::x27,
                },
            }),
            Op::Ret(Ret {}),
        ],
        r#"
       0: 0e 1b f0                        xconst8 x27, -16
       3: 12 41 6c                        xadd32 x1, x2, x27
       6: 29 01 08 02                     store64_offset8 x1, 8, x2
       a: 27 01 02                        store64 x1, x2
       d: 0b 01 02                        xmov x1, x2
      10: 12 00 04                        xadd32 x0, x0, x1
      13: 0b 01 02                        xmov x1, x2
      16: 25 01 02 08                     load64_offset8 x1, x2, 8
      1a: 22 01 02                        load64 x1, x2
      1d: 0e 1b 10                        xconst8 x27, 16
      20: 12 41 6c                        xadd32 x1, x2, x27
      23: 00                              ret

        "#,
    );
}
