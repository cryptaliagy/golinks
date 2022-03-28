/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"log"

	"github.com/spf13/cobra"
)

// clearCmd represents the clear command
var clearCmd = &cobra.Command{
	Use:   "clear",
	Short: "Clears all the routes from the GoLinks server",
	Long: `Clears all the routes from the GoLinks server.

	This does not require that the server be running.`,
	Run: func(cmd *cobra.Command, args []string) {
		log.Println("Clearing routes:")

		db := makeProvider(cmd)

		err := db.ClearRoutes()
		if err != nil {
			log.Fatal("Failed to clear routes")
		} else {
			log.Println("Routes cleared successfully")
		}
	},
}

func init() {
	rootCmd.AddCommand(clearCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// clearCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// clearCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
