package tui

import "github.com/charmbracelet/lipgloss"

var (
	subtle    = lipgloss.AdaptiveColor{Light: "#D9DCCF", Dark: "#383838"}
	highlight = lipgloss.AdaptiveColor{Light: "#874BFD", Dark: "#7D56F4"}
	special   = lipgloss.AdaptiveColor{Light: "#43BF6D", Dark: "#73F59F"}

	checkMark = lipgloss.NewStyle().SetString("✓").
			Foreground(special).
			PaddingRight(1).
			String()

	noCheckMark = "  "

	ListCursor = func(line string, enabled bool) string {
		var left string
		if enabled {
			left = checkMark
		} else {
			left = noCheckMark
		}

		return left + lipgloss.NewStyle().
			Foreground(lipgloss.Color("#569cd6")).Render("> "+line)
	}

	ListItem = func(line string, enabled bool) string {
		var left string
		if enabled {
			left = checkMark
		} else {
			left = noCheckMark
		}

		return left + lipgloss.NewStyle().
			PaddingLeft(2).Render(line)
	}
)