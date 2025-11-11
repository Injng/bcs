from pathlib import Path
import scrapy

class ClassesSpider(scrapy.Spider):
    name = "classes"

    async def start(self):
        yield scrapy.Request(url="https://classes.berkeley.edu/search/class?f[0]=term%3A8576&page=0", callback=self.parse)

    def parse(self, response):
        for c in response.css(".views-row"):
            code = c.css(".st--section-name::text").get()
            type = c.css(".st--section-code::text").get()
            count = c.css(".st--section-count::text").get()
            title = c.xpath("//div[@class='st--title']/h2/text()").get()
            yield {
                "code": code,
                "type": type,
                "count": count,
                "title": title,
                }
