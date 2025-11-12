from pathlib import Path
import scrapy

class ClassesSpider(scrapy.Spider):
    name = "classes"

    async def start(self):
        yield scrapy.Request(url="https://classes.berkeley.edu/search/class?f[0]=term%3A8576&page=0", callback=self.parse)

    def parse(self, response):
        for c in response.xpath("//div[@class='views-row']"):
            code = c.xpath(".//span[@class='st--section-name']/text()").get()
            type = c.xpath(".//span[@class='st--section-code']/text()").get()
            count = c.xpath(".//span[@class='st--section-count']/text()").get()
            title = c.xpath(".//div[@class='st--title']/h2/text()").get()
            subtitle = c.xpath(".//div[@class='st--subtitle']/text()").get()
            special = c.xpath(".//div[@class='st--special-title']/text()").get()
            link = c.xpath(".//div[@class='st--section-name-wraper']/a/@href").get()
            item = {
                "code": code,
                "type": type,
                "count": count,
                "title": title,
                "subtitle": subtitle or "",
                "special": special or "",
                "link": "https://classes.berkeley.edu" + link,
                }
            yield scrapy.Request(url=item["link"],
                                 callback=self.parse_class,
                                 cb_kwargs={'item': item})

        # next_page_xpath = "//li[@class='pager__item pager__item--next']/a/@href"
        # next_page = response.xpath(next_page_xpath).get()
        # if next_page is not None:
        #     next_page = response.urljoin(next_page)
        #    yield scrapy.Request(next_page, callback=self.parse)
                
    def parse_class(self, response, item):
        # xpath selectors
        location_xpath = "normalize-space(//div[@class='sf--location']/a/text())"
        id_xpath = "normalize-space(//span[text()='Class #:']/parent::div/text()[last()])"
        units_xpath = "normalize-space(//span[text()='Units:']/parent::div/text()[last()])"
        course_des_xpath = "//section[@id='section-course-description']/div/text()"
        class_des_xpath = "//section[@id='section-class-description']/div/text()"
        capacity_xpath = "normalize-space(//strong[text()='Capacity:']/parent::div/text())"
        enrolled_xpath = "normalize-space(//strong[text()='Enrolled:']/parent::div/text())"
        details_xpath = "//h3[text()='Current Enrollment']/parent::div/div[@class='details']"
        num_xpath = "./span/text()"
        classification_xpath = "normalize-space(./text()[last()])"

        # get info with xpath selectors
        location = response.xpath(location_xpath).get()
        id = response.xpath(id_xpath).get()
        units = response.xpath(units_xpath).get()
        course_des = response.xpath(course_des_xpath).get()
        class_des = response.xpath(class_des_xpath).get()
        capacity = response.xpath(capacity_xpath).get()
        enrolled = response.xpath(enrolled_xpath).get()
        details = response.xpath(details_xpath)
        seats = {}
        for s in details:
            num = int(s.xpath(num_xpath).get())
            classification = s.xpath(classification_xpath).get()
            seats[classification] = num

        # update dictionary with new info
        item["location"] = location or ""
        item["id"] = id
        item["units"] = units
        item["course-description"] = course_des or ""
        item["class-description"] = class_des or ""
        item["capacity"] = capacity
        item["enrolled"] = enrolled
        item["seats"] = seats
        yield item
