package factorio

import "sort"

type SortModsBy func(p1, p2 *Mod) bool

func (by SortModsBy) Sort(mods []Mod) {
	ms := &modSorter{
		mods: mods,
		by:   by,
	}
	sort.Sort(ms)
}

type modSorter struct {
	mods []Mod
	by   func(p1, p2 *Mod) bool
}

func (s *modSorter) Len() int {
	return len(s.mods)
}

func (s *modSorter) Swap(i, j int) {
	s.mods[i], s.mods[j] = s.mods[j], s.mods[i]
}

func (s *modSorter) Less(i, j int) bool {
	return s.by(&s.mods[i], &s.mods[j])
}
