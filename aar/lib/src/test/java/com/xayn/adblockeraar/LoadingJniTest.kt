package com.xayn.adblockeraar

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import java.io.File
import java.io.FileInputStream

class LoadingJniTest {

    @Test
    fun `Expect to create a new Engine`() {
        assertNotNull(
            Adblock.INSTANCE.createEngine(
                "-advertisement-icon.\n" +
                        "-advertisement-management\n" +
                        "-advertisement.\n" +
                        "-advertisement/script.\n" +
                        "@@good-advertisement\n"
            )
        )
    }

    @Test
    fun `Expect to match a rule when path of usr is matching`() {
        val engine = Adblock.INSTANCE.createEngine(
            "-advertisement-icon.\n" +
                    "-advertisement-management\n" +
                    "-advertisement.\n" +
                    "-advertisement/script.\n" +
                    "@@good-advertisement\n"
        )

        val match = engine.match(
            "http://example.com/-advertisement-icon.",
            "http://example.com/helloworld",
            "image"
        )
        assertEquals(true, match.isMatched)
        assertEquals(false, match.isImportant)
        assertEquals(false, match.isException)
    }

    @Test
    fun `Expect to not match any rule`() {
        val engine = Adblock.INSTANCE.createEngine(
            "-advertisement-icon.\n" +
                    "-advertisement-management\n" +
                    "-advertisement.\n" +
                    "-advertisement/script.\n" +
                    "@@good-advertisement\n"
        )

        val match = engine.match(
            "http://example.com/-jiberish.gif",
            "http://example.com/helloworld",
            "image"
        )
        assertEquals(false, match.isMatched)
        assertEquals(false, match.isImportant)
        assertEquals(false, match.isException)
    }

    @Test(expected = RuntimeException::class)
    fun `After finalizing an Engine, any further match will crash`() {
        val engine = Adblock.INSTANCE.createEngine(
            "-advertisement-icon.\n" +
                    "-advertisement-management\n" +
                    "-advertisement.\n" +
                    "-advertisement/script.\n" +
                    "@@good-advertisement\n"
        )

        engine.destroy()

        engine.match(
            "",
            "",
            ""
        )
    }

    @Test
    fun `Deserialize engine from byte array`() {
        val engine = Adblock.INSTANCE.createEngine()
        val file = File("src/test/rs-ABPFilterParserData.dat")

        engine.deserialize(FileInputStream(file))
        val matchPosiitive = engine.match(
            "https://www.googletagmanager.com/gtm.js?id=GTM-5LC93J" ,"https://www.guildwars2.com/en/", "script"
        )

        val matchNegative = engine.match(
            "https://www.google.com" ,"https://www.google.com", "script"
        )

        assertEquals(true, matchPosiitive.isMatched)
        assertEquals(false, matchNegative.isMatched)
    }

    @Test
    fun `Deserialize global engine from file`() {
        val engine = Adblock.INSTANCE.createEngine()
        val file = File("src/test/rs-ABPFilterParserData.dat")

        engine.deserialize(file.absolutePath)

        val matchPosiitive = engine.match(
            "https://www.googletagmanager.com/gtm.js?id=GTM-5LC93J" ,"https://www.guildwars2.com/en/", "script"
        )

        val matchNegative = engine. match(
            "https://www.google.com" ,"https://www.google.com", "script"
        )

        assertEquals(true, matchPosiitive.isMatched)
        assertEquals(false, matchNegative.isMatched)
    }

    @Test
    fun `Deserialize regional engine from file`() {
        val engine = Adblock.INSTANCE.createEngine()
        val file = File("src/test/rs-adblock_de.dat")

        val deserialize = engine.deserialize(file.absolutePath)

        val matchNegative = engine. match(
                "https://www.google.com" ,"https://www.google.com", "script"
        )

        assertEquals("Engine is not initialized successful!",true, deserialize, );
        assertEquals(false, matchNegative.isMatched)
    }

    @Test(expected = Exception::class)
    fun `Deserialize engine from file that does not exist should throw`() {
        val engine = Adblock.INSTANCE.createEngine()

        engine.deserialize("bla.dat")
    }
}