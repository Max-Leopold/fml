package modList

import (
	"log"
	"path/filepath"

	"github.com/Max-Leopold/factorio-mod-loader/internal/factorio"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
)

type item struct {
	factorio.Mod
	Enabled bool
}

type replaceMsg *item

func (i item) FilterValue() string { return i.Title }
func (i *item) ToggleEnable() tea.Msg {
	i.Enabled = !i.Enabled

	return replaceMsg(i)
}


type bubbleMod struct {
	list list.Model
}

func NewBubbleMod() bubbleMod {
	req := factorio.NewModsRequest()
	req.PageSize = "max"
	items := modsToBubbleMods(req.Execute())
	list := list.New(items, newItemDelegate(), 0, 0)
	list.Title = "Factorio Mods"

	return bubbleMod{
		list: list,
	}
}

func modsToBubbleMods(mods []factorio.Mod) []list.Item {
	modConfigPath, err := filepath.Abs("mod-list.json")
	if err != nil {
		log.Fatal(err)
	}
	config := factorio.GetModConfig(modConfigPath)
	configMap := make(map[string]bool)
	for i := 0; i < len(config.Mods); i += 1 {
		configMap[config.Mods[i].Name] = config.Mods[i].Enabled
	}

	items := make([]list.Item, len(mods))
	for i, v := range mods {
		_, enabled := configMap[v.Name]
		items[i] = item{
			Mod:     v,
			Enabled: enabled,
		}
	}

	return items
}
