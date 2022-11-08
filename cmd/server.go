/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>
*/
package cmd

import (
	"log"
	"os"

	"github.com/joho/godotenv"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"github.com/taliamax/golinks/server"
)

// serverCmd represents the server command
var serverCmd = &cobra.Command{
	Use:   "server",
	Short: "Starts up the redirect server",
	Long:  `Starts up the GoLinks redirect server.`,
	Run: func(cmd *cobra.Command, args []string) {
		cmd.Flags().Parse(args)

		if cmd.Flag("env-file").Value.String() != "" {
			godotenv.Load(cmd.Flag("env-file").Value.String())
		}

		config := server.ServerConfig{
			Port:    viper.GetString("port"),
			IsDebug: viper.GetBool("debug"),
			Key:     os.Getenv("GOLINKS_SECRET_KEY"),
		}

		dbFile := viper.GetString("database")

		engine := server.InitSQLiteServer(&config, dbFile)

		log.Println("Starting server on port:", config.Port)
		log.Println("Using database:", dbFile)

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

	viper.BindPFlag("port", serverCmd.Flags().Lookup("port"))
	viper.BindPFlag("debug", serverCmd.Flags().Lookup("debug"))
}
