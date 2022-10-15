package main

import (
	"fmt"

	"github.com/Max-Leopold/factorio-mod-loader/internal/tui/mod_list"
	tea "github.com/charmbracelet/bubbletea"
)

func main() {
	p := tea.NewProgram(modList.NewBubbleMod() , tea.WithAltScreen())

	if err := p.Start(); err != nil {
		fmt.Println("Error running program:", err)
	}
}
