package modList

import (
	"fmt"
	"io"

	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type itemDelegate struct{}

func (d itemDelegate) Height() int                               { return 1 }
func (d itemDelegate) Spacing() int                              { return 0 }
func (d itemDelegate) Update(msg tea.Msg, m *list.Model) tea.Cmd { return nil }
func (d itemDelegate) Render(w io.Writer, m list.Model, index int, listItem list.Item) {
	mod, ok := listItem.(item)
	if !ok {
		return
	}

	line := mod.Title

	if index == m.Index() {
		line = lipgloss.NewStyle().
			PaddingLeft(2).
			Foreground(lipgloss.Color("#569cd6")).Render("> " + line)
	} else {
		line = lipgloss.NewStyle().
			PaddingLeft(4).Render(line)
	}

	fmt.Fprint(w, line)
}
