package modList

import (
	"github.com/Max-Leopold/factorio-mod-loader/internal/factorio"
	"github.com/charmbracelet/bubbles/list"
)

type item factorio.Mod

func (i item) FilterValue() string { return i.Title }

type bubbleMod struct {
	list list.Model
}

func NewBubbleMod() bubbleMod {
	req := factorio.NewModsRequest()
	req.PageSize = "max"
	items := modsToBubbleMods(req.Execute())
	list := list.New(items, itemDelegate{}, 0, 0)
	list.Title = "Factorio Mods"

	return bubbleMod{
		list: list,
	}
}

func modsToBubbleMods(mods []factorio.Mod) []list.Item {
	items := make([]list.Item, len(mods))
	for i, v := range mods {
		items[i] = item(v)
	}

	return items
}
