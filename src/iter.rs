/*!
Doing the iteration.
*/

use crate::cx::Cx;

#[derive(Clone, Debug, PartialEq)]
pub enum IterParams {
    Mandlebrot,
    PseudoMandlebrot(Cx, Cx),
    Polynomial(Vec<Cx>)
}