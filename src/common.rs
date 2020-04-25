//! Common types and traits used for any puzzle.

use std::fmt::Debug;
use std::ops::{Add, Mul, Neg};

pub use traits::*;

/// All the traits from this module, for easy importing.
pub mod traits {
    use super::*;

    /// A twisty puzzle.
    pub trait PuzzleTrait: 'static + Debug + Default + Clone + Eq {
        /// The location of a piece of the puzzle.
        type Piece: PieceTrait<Self>;
        /// The location of a sticker of the puzzle.
        type Sticker: StickerTrait<Self>;
        /// The location of a face of the puzzle.
        type Face: FaceTrait<Self>;
        /// A twist that can be applied to the puzzle.
        type Twist: TwistTrait<Self>;
        /// An orientation for a puzzle piece, or a rotation that can be applied
        /// to an orientation.
        type Orientation: OrientationTrait<Self>;

        /// Returns a new solved puzzle in the default orientation.
        fn new() -> Self {
            Self::default()
        }
        /// Returns a reference to the piece at the given location.
        fn get_piece(&self, pos: Self::Piece) -> &Self::Orientation;
        /// Returns a mutable reference to the piece at the given location.
        fn get_piece_mut(&mut self, pos: Self::Piece) -> &mut Self::Orientation;
        /// Returns the face where the sticker at the given location belongs
        /// (i.e. corresponding to its color).
        fn get_sticker(&self, pos: Self::Sticker) -> Self::Face;
        /// Swaps two pieces on the puzzle by rotating the first through the
        /// given rotation and rotating the second in the reverse direction.
        fn swap(&mut self, pos1: Self::Piece, pos2: Self::Piece, rot: Self::Orientation) {
            let tmp = *self.get_piece(pos1);
            *self.get_piece_mut(pos1) = rot * *self.get_piece(pos2);
            *self.get_piece_mut(pos2) = rot.rev() * tmp;
        }
        /// Cycles pieces using the given starting piece and the given rotation.
        fn cycle(&mut self, start: Self::Piece, rot: Self::Orientation) {
            let rot = rot.rev();
            let mut prev = start;
            loop {
                let current = rot * prev;
                if current == start {
                    break;
                }
                self.swap(current, prev, rot);
                prev = current;
            }
        }
        /// Applies a twist to this puzzle.
        fn twist(&mut self, twist: Self::Twist) {
            for initial in twist.initial_pieces() {
                self.cycle(initial, twist.rotation())
            }
        }
    }

    /// The location of a piece in a twisty puzzle.
    pub trait PieceTrait<P: PuzzleTrait>: Debug + Copy + Eq {
        /// Returns the number of stickers on this piece (i.e. the length of
        /// self.stickers()).
        fn sticker_count(self) -> usize;
        /// Returns an iterator over all the stickers on this piece.
        fn stickers(self) -> Box<dyn Iterator<Item = P::Sticker> + 'static>;
        /// Returns an iterator over all the pieces in this puzzle.
        fn iter() -> Box<dyn Iterator<Item = Self>>;
    }

    /// The location of a sticker in a twisty puzzle.
    pub trait StickerTrait<P: 'static + PuzzleTrait>: Debug + Copy + Eq {
        /// Returns the piece that this sticker is on.
        fn piece(self) -> P::Piece;
        /// Returns the face that this sticker is on.
        fn face(self) -> P::Face;
        /// Returns the 3D vertices that this sticker is composed of.
        fn verts(self) -> [[f32; 3]; 4];
        /// Returns an iterator over all the stickers on this puzzle.
        fn iter() -> Box<dyn Iterator<Item = P::Sticker>> {
            Box::new(P::Piece::iter().flat_map(P::Piece::stickers))
        }
    }

    /// A face of a twisty puzzle.
    pub trait FaceTrait<P: PuzzleTrait>: Debug + Copy + Eq {
        /// The number of faces on this puzzle.
        const COUNT: usize;

        /// Returns a unique number corresponding to this face in the range
        /// 0..Self::COUNT.
        fn idx(self) -> usize;
        /// Returns an iterator over all the stickers on this face.
        fn stickers(self) -> Box<dyn Iterator<Item = P::Sticker> + 'static>;
        /// Returns an iterator over all the faces on this puzzle.
        fn iter() -> Box<dyn Iterator<Item = P::Face>>;
    }

    /// A twist that can be applied to a twisty puzzle.
    pub trait TwistTrait<P: PuzzleTrait>: 'static + Debug + Copy + Eq + From<P::Sticker> {
        /// Returns the orientation that would result from applying this twist
        /// to a piece in the default orientation.
        fn rotation(self) -> P::Orientation;
        /// Returns the reverse of this twist.
        #[must_use]
        fn rev(self) -> Self;
        /// Returns a list of pieces that are the "initial" pieces used to
        /// generate the full list of affected pieces; i.e. self.pieces() is
        /// generated by applying the generator function self.orientation() on
        /// the set of pieces returned by this function.
        ///
        /// Behavior of other methods is undefined if this function returns
        /// multiple pieces that are in the same generated set.
        fn initial_pieces(self) -> Vec<P::Piece>;
        /// Returns an iterator over the pieces affected by this twist.
        fn pieces(self) -> Box<dyn Iterator<Item = P::Piece>> {
            let initial_pieces = self.initial_pieces();
            let rot = self.rotation();
            Box::new(initial_pieces.into_iter().flat_map(move |start| {
                // Include the first piece.
                std::iter::once(start).chain(
                    // Keep reorienting that piece ...
                    std::iter::successors(Some(rot * start), move |&prev| Some(rot * prev))
                        // ... until we get back where we started.
                        .take_while(move |&x| x != start),
                )
            }))
        }
        /// Returns an iterator over the stickers affected by this twist.
        fn stickers(self) -> Box<dyn Iterator<Item = P::Sticker>> {
            Box::new(self.pieces().flat_map(P::Piece::stickers))
        }
    }

    /// An orientation for a piece of a twisty puzzle, relative to some default.
    pub trait OrientationTrait<P: PuzzleTrait>:
        Debug + Default + Copy + Eq + Mul<Self, Output = Self> + Mul<P::Piece, Output = P::Piece>
    {
        /// Reverses this orientation.
        #[must_use]
        fn rev(self) -> Self;
    }
}

/// Positive, negative, or zero.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sign {
    /// Negative.
    Neg = -1,
    /// Zero.
    Zero = 0,
    /// Positive.
    Pos = 1,
}
impl Default for Sign {
    fn default() -> Self {
        Self::Zero
    }
}
impl From<TwistDirection> for Sign {
    fn from(direction: TwistDirection) -> Self {
        match direction {
            TwistDirection::CW => Self::Pos,
            TwistDirection::CCW => Self::Neg,
        }
    }
}
impl Neg for Sign {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Self::Neg => Self::Pos,
            Self::Zero => Self::Zero,
            Self::Pos => Self::Neg,
        }
    }
}
impl Mul<Sign> for Sign {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        match self {
            Self::Neg => -rhs,
            Self::Zero => Self::Zero,
            Self::Pos => rhs,
        }
    }
}
impl Add<Sign> for Sign {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match self {
            Self::Neg => match rhs {
                Self::Neg => panic!("Too negative"),
                Self::Zero => Self::Neg,
                Self::Pos => Self::Zero,
            },
            Self::Zero => rhs,
            Self::Pos => match rhs {
                Self::Neg => Self::Zero,
                Self::Zero => Self::Pos,
                Self::Pos => panic!("Too positive"),
            },
        }
    }
}
impl Sign {
    /// Returns an integer representation of this sign (either -1, 0, or 1).
    pub fn int(self) -> isize {
        match self {
            Self::Neg => -1,
            Self::Zero => 0,
            Self::Pos => 1,
        }
    }
    /// Returns a floating-point representation of this sign (either -1.0, 0.0,
    /// or 1.0).
    pub fn float(self) -> f32 {
        self.int() as f32
    }
    /// Returns the absolute value of the integer representation of this sign (either 0 or 1).
    pub fn abs(self) -> usize {
        match self {
            Self::Neg | Self::Pos => 1,
            Self::Zero => 0,
        }
    }
    /// Returns true if this is Sign::Zero or false otherwise.
    pub fn is_zero(self) -> bool {
        self == Self::Zero
    }
    /// Returns false if this is Sign::Zero or true otherwise.
    pub fn is_nonzero(self) -> bool {
        self != Self::Zero
    }
    /// Returns an iterator over all Sign variants.
    pub fn iter() -> impl Iterator<Item = &'static Self> {
        [Self::Neg, Self::Zero, Self::Pos].iter()
    }
}

/// A rotation direction; clockwise or counterclockwise.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TwistDirection {
    /// Clockwise.
    CW,
    /// Counterclockwise.
    CCW,
}
impl Default for TwistDirection {
    fn default() -> Self {
        Self::CW
    }
}
impl TwistDirection {
    /// Returns the reverse direction.
    #[must_use]
    pub fn rev(self) -> Self {
        match self {
            Self::CW => Self::CCW,
            Self::CCW => Self::CW,
        }
    }
}
