package modList

import (
	"log"
	"path/filepath"

	"github.com/Max-Leopold/factorio-mod-loader/factorio"
	"github.com/Max-Leopold/factorio-mod-loader/factorio/config"
	"github.com/Max-Leopold/factorio-mod-loader/factorio/requests"
	tea "github.com/charmbracelet/bubbletea"
)

type item struct {
	factorio.Mod
	Enabled bool
}

type listReplaceMsg *item

func createListReplaceMsg(item *item) tea.Cmd {
	return func() tea.Msg {
		return listReplaceMsg(item)
	}
}

type statusListStatusMsg string

func createListStatusMsg (status string) tea.Cmd {
	return func() tea.Msg {
		return statusListStatusMsg(status)
	}
}

func (i item) FilterValue() string { return i.Title }
func (i *item) ToggleEnable() tea.Cmd {
	i.Enabled = !i.Enabled

	var statusMessage string
	if i.Enabled {
		statusMessage = "Enabled " + i.Title
	} else {
		statusMessage = "Disabled " + i.Title
	}

	return tea.Batch(createListReplaceMsg(i), createListStatusMsg(statusMessage))
}


type bubbleMod struct {
	list          Model
}

func NewBubbleMod() bubbleMod {
	req := requests.NewModsRequest()
	req.PageSize = "max"
	items := modsToBubbleMods(req.Execute())
	list := New(*items, NewModItemDelegate(), 0, 0)
	list.Title = "Factorio Mods"

	return bubbleMod{
		list: list,
	}
}

func modsToBubbleMods(mods *[]factorio.Mod) *[]Item {
	modConfigPath, err := filepath.Abs("mod-list.json")
	if err != nil {
		log.Fatal(err)
	}
	config := config.GetModConfig(modConfigPath)
	configMap := make(map[string]bool)
	for i := 0; i < len(config.Mods); i += 1 {
		configMap[config.Mods[i].Name] = config.Mods[i].Enabled
	}

	items := make([]Item, len(*mods))
	for i, v := range *mods {
		_, enabled := configMap[v.Name]
		items[i] = item{
			Mod:     v,
			Enabled: enabled,
		}
	}

	return &items
}
