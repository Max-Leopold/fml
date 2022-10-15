package modList

import (
	"fmt"
	"io"

	"github.com/Max-Leopold/factorio-mod-loader/internal/tui"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
)

type itemDelegate struct{}

func (d itemDelegate) Height() int                               { return 1 }
func (d itemDelegate) Spacing() int                              { return 0 }
func (d itemDelegate) Update(msg tea.Msg, m *list.Model) tea.Cmd { return nil }
func (d itemDelegate) Render(w io.Writer, m list.Model, index int, listItem list.Item) {
	item, ok := listItem.(item)
	if !ok {
		return
	}

	line := item.Title

	if index == m.Index() {
		line = tui.ListCursor(line, item.Enabled)
	} else {
		line = tui.ListItem(line, item.Enabled)
	}

	fmt.Fprint(w, line)
}
