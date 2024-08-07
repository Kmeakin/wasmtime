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
    let x0 = XReg::new(0).unwrap();
    let x1 = XReg::new(1).unwrap();
    let x27 = XReg::new(27).unwrap();

    assert_disas(
        &[
            // Prologue.
            Op::Xconst8(Xconst8 {
                dst: x27,
                imm: -16i8,
            }),
            Op::Xadd32(Xadd32 {
                operands: BinaryOperands {
                    dst: XReg::SP,
                    src1: XReg::SP,
                    src2: x27,
                },
            }),
            Op::Store64Offset8(Store64Offset8 {
                ptr: XReg::SP,
                offset: 8,
                src: XReg::LR,
            }),
            Op::Store64(Store64 {
                ptr: XReg::SP,
                src: XReg::FP,
            }),
            Op::Xmov(Xmov {
                dst: XReg::FP,
                src: XReg::SP,
            }),
            // Function body.
            Op::Xadd32(Xadd32 {
                operands: BinaryOperands {
                    dst: x0,
                    src1: x0,
                    src2: x1,
                },
            }),
            // Epilogue.
            Op::Xmov(Xmov {
                dst: XReg::SP,
                src: XReg::FP,
            }),
            Op::Load64Offset8(Load64Offset8 {
                dst: XReg::LR,
                ptr: XReg::SP,
                offset: 8,
            }),
            Op::Load64(Load64 {
                dst: XReg::FP,
                ptr: XReg::SP,
            }),
            Op::Xconst8(Xconst8 { dst: x27, imm: 16 }),
            Op::Xadd32(Xadd32 {
                operands: BinaryOperands {
                    dst: XReg::SP,
                    src1: XReg::SP,
                    src2: x27,
                },
            }),
            Op::Ret(Ret {}),
        ],
        r#"
       0: 0e 1b f0                        xconst8 sp, -16
       3: 12 7b 6f                        xadd32 sp, sp, sp
       6: 29 1b 08 1c                     store64_offset8 sp, 8, lr
       a: 27 1b 1d                        store64 sp, fp
       d: 0b 1d 1b                        xmov fp, sp
      10: 12 00 04                        xadd32 x0, x0, x1
      13: 0b 1b 1d                        xmov sp, fp
      16: 25 1c 1b 08                     load64_offset8 lr, sp, 8
      1a: 22 1d 1b                        load64 fp, sp
      1d: 0e 1b 10                        xconst8 sp, 16
      20: 12 7b 6f                        xadd32 sp, sp, sp
      23: 00                              ret
        "#,
    );
}
