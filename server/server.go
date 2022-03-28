package server

import (
	"log"
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
)

var provider DatabaseProvider

func InitSQLiteServer(config *ServerConfig, databaseFile string) *gin.Engine {
	log.Println("Initializing database...")
	provider = &SQLiteServerConfig{
		ServerConfig: *config,
	}

	provider.InitDB(databaseFile)

	log.Println("Creating server...")

	if config.IsDebug {
		gin.SetMode(gin.DebugMode)
	} else {
		gin.SetMode(gin.ReleaseMode)
	}

	r := gin.Default()

	log.Println("Setting up routes...")

	r.SetTrustedProxies(nil)
	r.GET("/heartbeat", heartbeat)
	r.GET("/:link", doRedirect)
	r.GET("/which/:link", getLink)

	private := r.Group("/route")
	private.Use(Authentication)
	{

		private.GET("/all", getAllRoutes)
		private.POST("/:link", addRoute)
		private.DELETE("/:link", removeRoute)
		private.PUT("/:link", updateRoute)
	}

	return r
}

func Authentication(c *gin.Context) {
	auth_header := c.Request.Header.Get("Authorization")

	if auth_header == "" {
		c.IndentedJSON(http.StatusUnauthorized, gin.H{
			"ok":  false,
			"msg": "No Authorization header",
		})
		c.Abort()
		return
	}

	token := strings.TrimPrefix(auth_header, "Bearer ")

	if !provider.ValidateAuth(token) {
		c.IndentedJSON(http.StatusUnauthorized, gin.H{
			"ok":  false,
			"msg": "Invalid token",
		})
		c.Abort()
		return
	}

	c.Next()
}

func heartbeat(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{
		"status": "ok",
	})
}

func doRedirect(c *gin.Context) {
	link := c.Param("link")
	if link == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "link not specified",
			"ok":    false,
		})
		return
	}

	redirect, err := provider.GetRoute(link)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": err.Error(),
			"ok":    false,
		})
		return
	}

	c.Redirect(http.StatusTemporaryRedirect, redirect)
}

func getLink(c *gin.Context) {
	link := c.Param("link")
	if link == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "link not specified",
			"ok":    false,
		})
		return
	}

	redirect, err := provider.GetRoute(link)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": err.Error(),
			"ok":    false,
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"url": redirect,
		"ok":  true,
	})
}

func getAllRoutes(c *gin.Context) {
	routes := provider.GetAllRoutes()

	c.JSON(http.StatusOK, gin.H{
		"routes": routes,
		"ok":     true,
	})
}

func addRoute(c *gin.Context) {
	link := c.Param("link")

	json := struct{ url string }{}

	if err := c.ShouldBindJSON(&json); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": err.Error(),
			"ok":    false,
		})
		return
	}

	url := json.url
	if url == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "url not specified",
			"ok":    false,
		})
		return
	}

	err := provider.AddRoute(link, url)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": err.Error(),
			"ok":    false,
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"ok": true,
	})
}

func removeRoute(c *gin.Context) {
	link := c.Param("link")
	if link == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "link not specified",
			"ok":    false,
		})
		return
	}

	err := provider.RemoveRoute(link)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": err.Error(),
			"ok":    false,
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"ok": true,
	})
}

func updateRoute(c *gin.Context) {
	link := c.Param("link")
	if link == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "link not specified",
			"ok":    false,
		})
		return
	}

	json := struct{ url string }{}

	if err := c.ShouldBindJSON(&json); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": err.Error(),
			"ok":    false,
		})
		return
	}

	url := json.url
	if url == "" {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "url not specified",
			"ok":    false,
		})
		return
	}

	err := provider.UpdateRoute(link, url)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": err.Error(),
			"ok":    false,
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"ok": true,
	})
}
