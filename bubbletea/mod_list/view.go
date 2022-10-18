package modList

import (
	"github.com/charmbracelet/lipgloss"
)

var docStyle = lipgloss.NewStyle().Margin(1, 2)

func (b bubbleMod) View() string {
	return b.listView()
}

func (b bubbleMod) listView() string {
	return docStyle.Render(b.list.View())
}
