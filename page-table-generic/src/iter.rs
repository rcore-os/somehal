use super::{Access, PTEGeneric, PTEInfo, PageTableRef, TableGeneric};

pub struct TableIter<'a, 'b: 'a, P: TableGeneric, A: Access> {
    access: &'b A,
    idx_stack: [usize; 12],
    table_stack: [Option<PageTableRef<'a, P>>; 12],
    level: usize,
    max_level: usize,
    start_vaddr: *const u8,
}

impl<'a, 'b: 'a, P: TableGeneric, A: Access> TableIter<'a, 'b, P, A> {
    pub fn new(va: *const u8, root: PageTableRef<'a, P>, access: &'b A) -> Self {
        let mut table_stack = [const { None }; 12];
        let max_level = root.level();
        table_stack[max_level - 1] = Some(root);

        TableIter {
            idx_stack: [0; 12],
            table_stack,
            level: max_level,
            access,
            start_vaddr: va,
            max_level,
        }
    }

    fn idx(&self) -> usize {
        self.idx_stack[self.level - 1]
    }

    fn table(&self) -> PageTableRef<'a, P> {
        self.table_stack[self.level - 1].unwrap()
    }

    fn entries(&self) -> &[P::PTE] {
        self.table().as_slice(self.access)
    }

    fn pte(&self) -> Option<P::PTE> {
        let idx = self.idx();
        let entries = self.entries();
        if idx >= entries.len() {
            return None;
        }
        Some(entries[idx])
    }

    fn idx_next(&mut self, pte: P::PTE) {
        if pte.is_huge() || self.level == 1 || !pte.valid() {
            self.idx_stack[self.level - 1] += 1;
            if self.level < self.max_level && self.idx() >= P::TABLE_LEN {
                self.level += 1;
                self.idx_stack[self.level - 1] += 1;
            }
        } else {
            self.level -= 1;
            self.idx_stack[self.level - 1] = 0;
            self.table_stack[self.level - 1] =
                Some(PageTableRef::from_addr(pte.paddr(), self.level));
        }
    }

    fn vaddr(&self) -> *const u8 {
        unsafe {
            let mut offset = 0;

            for i in (0..12).rev() {
                if let Some(tb) = self.table_stack[i] {
                    offset += self.idx_stack[i] * tb.entry_size();
                }
            }

            self.start_vaddr.add(offset)
        }
    }
}

impl<'a, 'b: 'a, P: TableGeneric, A: Access> Iterator for TableIter<'a, 'b, P, A> {
    type Item = PTEInfo<P::PTE>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.pte() {
                Some(pte) => {
                    let out = if pte.valid() {
                        let vaddr = self.vaddr();
                        Some(PTEInfo {
                            level: self.level,
                            vaddr: (vaddr as usize).into(),
                            pte,
                        })
                    } else {
                        None
                    };
                    self.idx_next(pte);
                    if let Some(out) = out {
                        return Some(out);
                    }
                }
                None => {
                    if self.level == self.max_level {
                        return None;
                    } else {
                        self.level += 1;
                        self.idx_stack[self.level - 1] += 1;
                        continue;
                    }
                }
            };
        }
    }
}
