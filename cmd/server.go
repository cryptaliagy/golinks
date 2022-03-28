/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"log"
	"os"

	"github.com/joho/godotenv"
	"github.com/spf13/cobra"
	"github.com/taliamax/golinks/server"
)

// serverCmd represents the server command
var serverCmd = &cobra.Command{
	Use:   "server",
	Short: "Starts up the redirect server",
	Long:  `Starts up the GoLinks redirect server.`,
	Run: func(cmd *cobra.Command, args []string) {
		cmd.Flags().Parse(args)

		godotenv.Load(cmd.Flag("env-file").Value.String())
		config := server.ServerConfig{
			Port:    cmd.Flag("port").Value.String(),
			IsDebug: cmd.Flag("debug").Value.String() == "true",
			Key:     os.Getenv("GOLINKS_SECRET_KEY"),
		}

		dbFile, err := cmd.Flags().GetString("database")

		if err != nil {
			log.Fatal(err)
		}

		engine := server.InitSQLiteServer(&config, dbFile)

		log.Println("Starting server on port:", config.Port)

		engine.Run(":" + config.Port)
	},
}

func init() {
	rootCmd.AddCommand(serverCmd)

	serverCmd.
		Flags().
		StringP("port", "p", "8080", "port to listen on")

	serverCmd.
		Flags().
		BoolP("debug", "d", false, "debug mode")

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// serverCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// serverCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
