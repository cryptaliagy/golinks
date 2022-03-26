/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"log"

	"github.com/spf13/cobra"
)

// addCmd represents the add command
var addCmd = &cobra.Command{
	Use:   "add [tag] [url]",
	Short: "Adds a route to the GoLinks server",
	Long: `Adds a route to the GoLinks server.

	This does not require that the server be running.`,
	Args: cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		cmd.Flags().Parse(args)
		log.Println("Adding route:", args[0], args[1])
		err := addRoute(args[0], args[1])
		if err != nil {
			log.Fatal("Failed to add route")
		} else {
			log.Println("Route added successfully")
		}
	},
}

func init() {
	rootCmd.AddCommand(addCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// addCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// addCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
