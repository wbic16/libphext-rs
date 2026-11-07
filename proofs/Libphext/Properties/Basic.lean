module
public import Aeneas
public import Libphext.Code.Funs
@[expose] public section

open Aeneas Aeneas.Std Result

/-! First kernel-checked specifications for libphext's translated model.
    Each theorem is stated against the Aeneas-generated definition of the
    same name in Libphext.Code.Funs. -/

/-- Advancing a coordinate component adds exactly one (when no overflow). -/
theorem advance_coordinate_spec (i : Std.Usize) (hi : i.val < Std.Usize.max) :
    Libphext.phext.Coordinate.advance_coordinate i ⦃ z => z.val = i.val + 1 ⦄ := by
  unfold Libphext.phext.Coordinate.advance_coordinate
  step*

/-- The default coordinate is 1 in all nine components. -/
theorem default_coordinate_spec :
    Libphext.phext.default_coordinate ⦃ c =>
      c.z.library = 1#usize ∧ c.z.shelf = 1#usize ∧ c.z.series = 1#usize ∧
      c.y.collection = 1#usize ∧ c.y.volume = 1#usize ∧ c.y.book = 1#usize ∧
      c.x.chapter = 1#usize ∧ c.x.«section» = 1#usize ∧ c.x.scroll = 1#usize ⦄ := by
  unfold Libphext.phext.default_coordinate
  step*

/-- A byte is a phext break iff it equals one of the ten break bytes. -/
theorem is_phext_break_spec (b : Std.U8) :
    Libphext.phext.is_phext_break b ⦃ z => z = true ↔
      b = Libphext.phext.LINE_BREAK_U8 ∨ b = Libphext.phext.SCROLL_BREAK_U8 ∨
      b = Libphext.phext.SECTION_BREAK_U8 ∨ b = Libphext.phext.CHAPTER_BREAK_U8 ∨
      b = Libphext.phext.BOOK_BREAK_U8 ∨ b = Libphext.phext.VOLUME_BREAK_U8 ∨
      b = Libphext.phext.COLLECTION_BREAK_U8 ∨ b = Libphext.phext.SERIES_BREAK_U8 ∨
      b = Libphext.phext.SHELF_BREAK_U8 ∨ b = Libphext.phext.LIBRARY_BREAK_U8 ⦄ := by
  unfold Libphext.phext.is_phext_break
  step*
