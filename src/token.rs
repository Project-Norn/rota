#[derive(Eq, PartialEq, Debug)]
pub enum Token {
    Integer { value: u32 },
    Ident { name: String },

    Commna,
    Colon,

    Push,
    Pop,
    Add,
    Sub,
    IMul,
    IDiv,
    Xor,
    Ret,
    Mov,
    Jmp,
    And,
    Or,
    Cmp,
    Sete,

    Eax,
    Ecx,
    Edx,
    Ebx,
    Esp,
    Ebp,
    Esi,
    Edi,

    Al,
    Cl,
    Dl,
    Bl,

    EOF,
}
