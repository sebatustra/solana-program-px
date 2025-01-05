
use solana_program::program_error::ProgramError;
use crate::errors::CustomError;

pub fn fixed_point_divide_checked(a: u64, b: u64) -> Result<u64, ProgramError> {
    if b == 0 {
        return Err(CustomError::ArithmeticError.into());
    }

    const SCALE: u64 = 1_000_000;

    let scaled_a = a.checked_mul(SCALE)
        .ok_or::<ProgramError>(CustomError::ArithmeticError.into())?;

    let result = scaled_a.checked_div(b)
    .ok_or::<ProgramError>(CustomError::ArithmeticError.into())?;

    Ok(result)
}