package modList

import (
	"fmt"
	"io"

	"github.com/Max-Leopold/factorio-mod-loader/tapioca"
	"github.com/charmbracelet/bubbles/key"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/muesli/reflow/truncate"
)

type ModItemDelegate struct{
	DelegateKeyMap delegateKeyMap
}

type delegateKeyMap struct {
	enable key.Binding
}

func NewModItemDelegate() ModItemDelegate {
	return ModItemDelegate{
		DelegateKeyMap: newItemDelegateKeyMap(),
	}
}

func newItemDelegateKeyMap() delegateKeyMap {
	return delegateKeyMap{
		enable: key.NewBinding(
			key.WithKeys("enter"),
			key.WithHelp("enter", "enable"),
		),
	}
}

// Height, Spacing, Update and Render implement the list.ItemDelegate interface
func (d ModItemDelegate) Height() int  { return 1 }
func (d ModItemDelegate) Spacing() int { return 0 }
func (d ModItemDelegate) Update(msg tea.Msg, l *Model) tea.Cmd {
	var cmd tea.Cmd

	switch msg := msg.(type) {
	case listReplaceMsg:
		// We can't edit an item directly, we can only modify a copy of it.
		// The workaround I used is to change the value on the copy and then replace the item
		// at the original index with the copy.
		// bubbles/list doesn't have the functionality to find the "actual" index of an item
		// when filtering and only returns the index in the filtered list.
		// So we have to find it ourselves, by iterating over all items to find the index of the
		// selected item in the filtered list in the list of all items.
		newItem := listReplaceMsg(msg)
		allItems := l.Items()
		var i int
		for index, elem := range allItems {
			if elem.Name == newItem.Name {
				i = index
				break
			}
		}
		cmd = tea.Batch(cmd, l.SetItem(i, newItem))
	case statusListStatusMsg:
		cmd = tea.Batch(cmd, l.NewStatusMessage(string(msg)))
	case tea.KeyMsg:
		switch {
		case key.Matches(msg, d.DelegateKeyMap.enable):
			// Toggle enable
			selectedItem := l.SelectedItem()
			cmd = tea.Batch(cmd, selectedItem.ToggleEnable())
		}
	}

	return cmd
}

func (d ModItemDelegate) Render(w io.Writer, m Model, index int, listItem *Item) {
	if m.Width() <= 0 {
		return
	}

	var title = truncate.StringWithTail(listItem.Title, uint(m.Width()), tapioca.Ellipsis)
	if index == m.Index() {
		title = tapioca.ListCursor(listItem.Title, listItem.Enabled)
	} else {
		title = tapioca.ListItem(listItem.Title, listItem.Enabled)
	}

	fmt.Fprintf(w, "%s", title)
	return
}

// ShortHelp and FullHelp implement the help.KeyMap interface
func (d ModItemDelegate) ShortHelp() []key.Binding {
	return []key.Binding{
		d.DelegateKeyMap.enable,
	}
}

func (d ModItemDelegate) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{
			d.DelegateKeyMap.enable,
		},
	}
}
