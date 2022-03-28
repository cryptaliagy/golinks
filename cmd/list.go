/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"log"

	"github.com/spf13/cobra"
)

// listCmd represents the list command
var listCmd = &cobra.Command{
	Use:   "list",
	Short: "Lists all the routes in the GoLinks server",
	Long: `Lists all the routes in the GoLinks server.

	This does not require that the server be running.`,
	Run: func(cmd *cobra.Command, args []string) {

		db := makeProvider(cmd)

		log.Println("Listing routes:")

		routes := db.GetAllRoutes()

		for tag, route := range routes {
			log.Println("\t", tag+":", route)
		}
	},
}

func init() {
	rootCmd.AddCommand(listCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// listCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// listCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
