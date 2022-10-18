package main

import (
	"fmt"
	"os"

	modList "github.com/Max-Leopold/factorio-mod-loader/bubbletea/mod_list"
	tea "github.com/charmbracelet/bubbletea"
)

func main() {
	f, err := tea.LogToFile("debug.log", "debug")
	if err != nil {
		fmt.Println("fatal:", err)
		os.Exit(1)
	}
	defer f.Close()

	p := tea.NewProgram(modList.NewBubbleMod(), tea.WithAltScreen())

	if err := p.Start(); err != nil {
		fmt.Println("Error running program:", err)
	}
}
