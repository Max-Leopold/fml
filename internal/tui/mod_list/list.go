package modList

import (
	"fmt"
	"io"

	"github.com/Max-Leopold/factorio-mod-loader/internal/tui"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/muesli/reflow/truncate"
)

type itemDelegate struct{}

func newItemDelegate() itemDelegate {
	return itemDelegate{}
}

func (d itemDelegate) Height() int  { return 1 }
func (d itemDelegate) Spacing() int { return 0 }
func (d itemDelegate) Update(msg tea.Msg, l *list.Model) tea.Cmd {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "enter":
			// We can't edit an item directly, we can only modify a copy of it.
			// The workaround I used is to change the value on the copy and then replace the item
			// at the original index with the copy.
			// bubbles/list doesn't have the functionality to find the "actual" index of an item
			// when filtering and only returns the index in the filtered list.
			// So we have to find it ourselves, by iterating over all items to find the index of the
			// selected item in the filtered list in the list of all items.
			selectedItem := l.SelectedItem().(item)
			allItems := l.Items()
			var i int
			for index, elem := range allItems {
				if elem.(item).Name == selectedItem.Name {
					i = index
					break
				}
			}

			selectedItem.ToggleEnable()
			return l.SetItem(i, selectedItem)
		}
	}

	return nil
}

func (d itemDelegate) Render(w io.Writer, m list.Model, index int, listItem list.Item) {
	var (
		mod item
		title string
	)

	if i, ok := listItem.(item); ok {
		mod = i
		title = i.Title
	} else {
		return
	}

	if m.Width() <= 0 {
		return
	}

	title = truncate.StringWithTail(mod.Title, uint(m.Width()), tui.Ellipsis)
	if index == m.Index() {
		title = tui.ListCursor(title, mod.Enabled)
	} else {
		title = tui.ListItem(title, mod.Enabled)
	}

	fmt.Fprintf(w, "%s", title)
	return
}
