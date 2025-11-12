from datetime import datetime
from pathlib import Path
import scrapy

class ClassesSpider(scrapy.Spider):
    name = "classes"

    async def start(self):
        yield scrapy.Request(url="https://classes.berkeley.edu/search/class?f[0]=term%3A8576&page=50", callback=self.parse)

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
                "class-type": type,
                "count": count,
                "title": title,
                "subtitle": subtitle or "",
                "special": special or "",
                "link": "https://classes.berkeley.edu" + link,
                }
            yield scrapy.Request(url=item["link"],
                                 callback=self.parse_class,
                                 cb_kwargs={'item': item})

        next_page_xpath = "//li[@class='pager__item pager__item--next']/a/@href"
        next_page = response.xpath(next_page_xpath).get()
        if next_page is not None:
            next_page = response.urljoin(next_page)
            yield scrapy.Request(next_page, callback=self.parse)
                
    def parse_class(self, response, item):
        # xpath selectors
        instr_xpath = "normalize-space(//div[@class='sf--instructors']/p/text()[last()])"
        days_xpath = "//div[@class='sf--meeting-days']/span[last()]/text()"
        time_xpath = "//div[@class='sf--meeting-time']/span[last()]/text()"
        location_xpath = "normalize-space(//div[@class='sf--location']/a/text())"
        id_xpath = "normalize-space(//span[text()='Class #:']/parent::div/text()[last()])"
        units_xpath = "normalize-space(//span[text()='Units:']/parent::div/text()[last()])"
        instruction_xpath = "//strong[text()='Instruction Mode:']/following::span/text()"
        course_des_xpath = "//section[@id='section-course-description']/div/text()"
        class_des_xpath = "//section[@id='section-class-description']/div/text()"
        capacity_xpath = "normalize-space(//strong[text()='Capacity:']/parent::div/text())"
        enrolled_xpath = "normalize-space(//strong[text()='Enrolled:']/parent::div/text())"
        wl_xpath = "normalize-space(//strong[text()='Waitlisted:']/parent::div/text())"
        wlmax_xpath = "normalize-space(//strong[text()='Waitlist Max:']/parent::div/text())"
        details_xpath = "//h3[text()='Current Enrollment']/parent::div/div[@class='details']"
        requirements_xpath = "//div[@title='General Requirements']/text()"
        num_xpath = "./span/text()"
        classification_xpath = "normalize-space(./text()[last()])"

        # get info with xpath selectors
        instructor = response.xpath(instr_xpath).get()
        days = response.xpath(days_xpath).get()
        time = response.xpath(time_xpath).get()
        if time: time = time.split("-")
        start_time = ""
        end_time = ""
        if time and len(time) == 1:
            time[0] = time[0].strip()
            start_time = datetime.strptime(time[0], "%I:%M %p").time().strftime("%H:%M")
        if time and len(time) == 2:
            time[1] = time[1].strip()
            end_time = datetime.strptime(time[1], "%I:%M %p").time().strftime("%H:%M")
        location = response.xpath(location_xpath).get()
        id = response.xpath(id_xpath).get()
        units = response.xpath(units_xpath).get()
        course_des = response.xpath(course_des_xpath).get()
        class_des = response.xpath(class_des_xpath).get()
        capacity = int(response.xpath(capacity_xpath).get())
        enrolled = int(response.xpath(enrolled_xpath).get())
        waitlist = int(response.xpath(wl_xpath).get())
        waitlist_max = int(response.xpath(wlmax_xpath).get())
        reqs = response.xpath(requirements_xpath).getall()
        details = response.xpath(details_xpath)
        seats = {}
        for s in details:
            num = int(s.xpath(num_xpath).get())
            classification = s.xpath(classification_xpath).get()
            seats[classification] = num

        # update dictionary with new info
        item["instructor"] = instructor or ""
        item["days"] = days or ""
        item["start"] = start_time or ""
        item["end"] = end_time or ""
        item["location"] = location or ""
        item["id"] = id
        item["units"] = units
        item["course-description"] = course_des or ""
        item["class-description"] = class_des or ""
        item["capacity"] = capacity
        item["enrolled"] = enrolled
        item["waitlist"] = waitlist
        item["waitlist-max"] = waitlist_max
        item["requirements"] = list(map(lambda x: " ".join(x.split()[1:]), reqs))
        item["seats"] = seats
        yield item
